//! CPU, memory, network and battery readings straight from Win32: no WMI, no performance
//! counters. Readings are only taken while something on screen asks for them (`system_watch`),
//! except the cheap power check that drives the charging live activity.

use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::Win32::{
    Foundation::FILETIME,
    NetworkManagement::{
        IpHelper::{FreeMibTable, GetIfTable2, MIB_IF_TABLE2},
        Ndis::IfOperStatusUp,
    },
    System::{
        Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS},
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        Threading::GetSystemTimes,
    },
};

static WATCHERS: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    cpu: f64,
    memory: f64,
    memory_used_gb: f64,
    memory_total_gb: f64,
    net_down: f64,
    net_up: f64,
    battery: Option<Battery>,
}

#[derive(Serialize, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    percent: u8,
    charging: bool,
    plugged: bool,
    saver: bool,
    seconds_left: Option<u32>,
}

fn filetime(ft: FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64
}

fn cpu_times() -> Option<(u64, u64)> {
    let (mut idle, mut kernel, mut user) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).ok()? };
    let idle = filetime(idle);
    // Kernel time includes idle time.
    Some((idle, filetime(kernel) + filetime(user)))
}

fn memory() -> (f64, f64, f64) {
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut status) }.is_err() || status.ullTotalPhys == 0 {
        return (0.0, 0.0, 0.0);
    }
    let gb = 1024.0 * 1024.0 * 1024.0;
    let total = status.ullTotalPhys as f64;
    let used = total - status.ullAvailPhys as f64;
    (used / total * 100.0, used / gb, total / gb)
}

/// Total bytes received and sent by physical interfaces that are up.
fn network_bytes() -> (u64, u64) {
    let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
    unsafe {
        if GetIfTable2(&mut table).is_err() || table.is_null() {
            return (0, 0);
        }
        let rows =
            std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        let (mut down, mut up) = (0u64, 0u64);
        for row in rows {
            // Bit 0 of InterfaceAndOperStatusFlags: HardwareInterface. Skips loopback, tunnels, filters.
            let hardware = row.InterfaceAndOperStatusFlags._bitfield & 1 != 0;
            if hardware && row.OperStatus == IfOperStatusUp {
                down += row.InOctets;
                up += row.OutOctets;
            }
        }
        FreeMibTable(table as *const core::ffi::c_void);
        (down, up)
    }
}

pub fn battery() -> Option<Battery> {
    let mut status = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut status).ok()? };
    // 128: no system battery; 255: unknown.
    if status.BatteryFlag & 128 != 0 || status.BatteryFlag == 255 || status.BatteryLifePercent > 100
    {
        return None;
    }
    Some(Battery {
        percent: status.BatteryLifePercent,
        charging: status.BatteryFlag & 8 != 0,
        plugged: status.ACLineStatus == 1,
        saver: status.SystemStatusFlag == 1,
        seconds_left: (status.BatteryLifeTime != u32::MAX).then_some(status.BatteryLifeTime),
    })
}

static LAST_BATTERY: Mutex<Option<Battery>> = Mutex::new(None);

pub fn start(app: AppHandle) {
    thread::Builder::new()
        .name("system".into())
        .spawn(move || {
            let mut previous_cpu = cpu_times();
            let mut previous_net = network_bytes();
            let mut previous_at = Instant::now();
            let mut power_tick = 0u32;
            loop {
                thread::sleep(Duration::from_millis(1000));
                power_tick += 1;
                if power_tick >= 5 {
                    power_tick = 0;
                    let now = battery();
                    let mut last = LAST_BATTERY.lock().expect("battery");
                    if *last != now {
                        let _ = app.emit("power://changed", &now);
                        *last = now;
                    }
                }
                if WATCHERS.load(Ordering::Relaxed) == 0 {
                    // Keep the baselines fresh so the first reading after a pause is not a long average.
                    previous_cpu = cpu_times();
                    previous_net = network_bytes();
                    previous_at = Instant::now();
                    continue;
                }
                let cpu_now = cpu_times();
                let cpu = match (previous_cpu, cpu_now) {
                    (Some((idle0, total0)), Some((idle1, total1))) if total1 > total0 => {
                        let total = (total1 - total0) as f64;
                        let idle = idle1.saturating_sub(idle0) as f64;
                        ((total - idle) / total * 100.0).clamp(0.0, 100.0)
                    }
                    _ => 0.0,
                };
                previous_cpu = cpu_now;
                let net_now = network_bytes();
                let seconds = previous_at.elapsed().as_secs_f64().max(0.001);
                previous_at = Instant::now();
                let net_down = net_now.0.saturating_sub(previous_net.0) as f64 / seconds;
                let net_up = net_now.1.saturating_sub(previous_net.1) as f64 / seconds;
                previous_net = net_now;
                let (memory, memory_used_gb, memory_total_gb) = memory();
                let stats = Stats {
                    cpu,
                    memory,
                    memory_used_gb,
                    memory_total_gb,
                    net_down,
                    net_up,
                    battery: battery(),
                };
                let _ = app.emit("system://stats", &stats);
            }
        })
        .expect("system thread");
}

/// Reference-counted: each view that shows readings calls watch(true) on mount and watch(false) on unmount.
#[tauri::command]
pub fn system_watch(on: bool) {
    if on {
        WATCHERS.fetch_add(1, Ordering::Relaxed);
    } else {
        let _ = WATCHERS.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
            Some(n.saturating_sub(1))
        });
    }
}

#[tauri::command]
pub fn power_get() -> Option<Battery> {
    battery()
}
