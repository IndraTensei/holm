//! System readings on macOS and Linux. Linux reads /proc and /sys (no library, no polling while
//! nothing shows the readings); macOS reports nothing yet. Same events and commands as the
//! Windows version in ../system.rs.

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

#[cfg(target_os = "linux")]
mod read {
    use super::Battery;
    use std::fs;

    /// (idle, total) jiffies from the first line of /proc/stat.
    pub fn cpu_times() -> Option<(u64, u64)> {
        let stat = fs::read_to_string("/proc/stat").ok()?;
        let line = stat.lines().next()?;
        let values: Vec<u64> = line
            .split_whitespace()
            .skip(1)
            .filter_map(|v| v.parse().ok())
            .collect();
        let idle = values.get(3)? + values.get(4).copied().unwrap_or(0);
        Some((idle, values.iter().sum()))
    }

    /// (percent used, used GB, total GB) from /proc/meminfo.
    pub fn memory() -> (f64, f64, f64) {
        let Ok(info) = fs::read_to_string("/proc/meminfo") else {
            return (0.0, 0.0, 0.0);
        };
        let field = |name: &str| {
            info.lines()
                .find(|l| l.starts_with(name))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0)
        };
        let total = field("MemTotal:");
        let available = field("MemAvailable:");
        if total <= 0.0 {
            return (0.0, 0.0, 0.0);
        }
        let used = total - available;
        let gb = 1024.0 * 1024.0;
        (used / total * 100.0, used / gb, total / gb)
    }

    /// (received, sent) bytes over every interface but loopback, from /proc/net/dev.
    pub fn network_bytes() -> (u64, u64) {
        let Ok(dev) = fs::read_to_string("/proc/net/dev") else {
            return (0, 0);
        };
        dev.lines()
            .skip(2)
            .filter_map(|line| {
                let (name, rest) = line.split_once(':')?;
                if name.trim() == "lo" {
                    return None;
                }
                let v: Vec<u64> = rest
                    .split_whitespace()
                    .filter_map(|x| x.parse().ok())
                    .collect();
                Some((*v.first()?, *v.get(8)?))
            })
            .fold((0, 0), |(r, s), (a, b)| (r + a, s + b))
    }

    /// The first battery under /sys/class/power_supply, if the machine has one.
    pub fn battery() -> Option<Battery> {
        let dir = fs::read_dir("/sys/class/power_supply").ok()?;
        for entry in dir.flatten() {
            let path = entry.path();
            let kind = fs::read_to_string(path.join("type")).unwrap_or_default();
            if kind.trim() != "Battery" {
                continue;
            }
            let percent: u8 = fs::read_to_string(path.join("capacity"))
                .ok()?
                .trim()
                .parse()
                .ok()?;
            let status = fs::read_to_string(path.join("status")).unwrap_or_default();
            let status = status.trim();
            return Some(Battery {
                percent: percent.min(100),
                charging: status == "Charging",
                plugged: status != "Discharging",
                saver: false,
                seconds_left: None,
            });
        }
        None
    }
}

#[cfg(not(target_os = "linux"))]
mod read {
    use super::Battery;

    pub fn cpu_times() -> Option<(u64, u64)> {
        None
    }
    pub fn memory() -> (f64, f64, f64) {
        (0.0, 0.0, 0.0)
    }
    pub fn network_bytes() -> (u64, u64) {
        (0, 0)
    }
    pub fn battery() -> Option<Battery> {
        None
    }
}

pub fn battery() -> Option<Battery> {
    read::battery()
}

static LAST_BATTERY: Mutex<Option<Battery>> = Mutex::new(None);

pub fn start(app: AppHandle) {
    thread::Builder::new()
        .name("system".into())
        .spawn(move || {
            let mut previous_cpu = read::cpu_times();
            let mut previous_net = read::network_bytes();
            let mut previous_at = Instant::now();
            let mut power_tick = 0u32;
            loop {
                thread::sleep(Duration::from_millis(1000));
                power_tick += 1;
                if power_tick >= 5 {
                    power_tick = 0;
                    let now = read::battery();
                    let mut last = LAST_BATTERY.lock().expect("battery");
                    if *last != now {
                        let _ = app.emit("power://changed", &now);
                        *last = now;
                    }
                }
                if WATCHERS.load(Ordering::Relaxed) == 0 {
                    previous_cpu = read::cpu_times();
                    previous_net = read::network_bytes();
                    previous_at = Instant::now();
                    continue;
                }
                let cpu_now = read::cpu_times();
                let cpu = match (previous_cpu, cpu_now) {
                    (Some((idle0, total0)), Some((idle1, total1))) if total1 > total0 => {
                        let total = (total1 - total0) as f64;
                        let idle = idle1.saturating_sub(idle0) as f64;
                        ((total - idle) / total * 100.0).clamp(0.0, 100.0)
                    }
                    _ => 0.0,
                };
                previous_cpu = cpu_now;
                let net_now = read::network_bytes();
                let seconds = previous_at.elapsed().as_secs_f64().max(0.001);
                previous_at = Instant::now();
                let net_down = net_now.0.saturating_sub(previous_net.0) as f64 / seconds;
                let net_up = net_now.1.saturating_sub(previous_net.1) as f64 / seconds;
                previous_net = net_now;
                let (memory, memory_used_gb, memory_total_gb) = read::memory();
                let stats = Stats {
                    cpu,
                    memory,
                    memory_used_gb,
                    memory_total_gb,
                    net_down,
                    net_up,
                    battery: read::battery(),
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
        #[allow(deprecated)]
        let _ = WATCHERS.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
            Some(n.saturating_sub(1))
        });
    }
}

#[tauri::command]
pub fn power_get() -> Option<Battery> {
    battery()
}
