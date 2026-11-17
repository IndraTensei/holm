//! Graphics: each GPU's load and memory, as Task Manager shows them.
//!
//! The adapters come from DXGI (name, memory sizes, LUID); their load and memory in use come
//! from Windows' GPU performance counters, which have no Win32 equivalent. A `Reader` exists only
//! while something on screen shows the readings (`system_watch`), and is dropped otherwise.
//!
//! Load is worked out as Task Manager does: per engine of an adapter (3D, copy, video decode, …)
//! the share of every process is added up, and the busiest engine is the GPU's load.

use std::collections::HashMap;

use serde::Serialize;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE},
        System::Performance::{
            PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData,
            PdhGetFormattedCounterArrayW, PdhOpenQueryW, PDH_FMT, PDH_FMT_COUNTERVALUE_ITEM_W,
            PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA,
        },
    },
};

/// PDH_FMT_NOCAP100: a process's share of an engine is not capped at 100 before adding up.
const NOCAP100: u32 = 0x8000;
const GB: f64 = 1024.0 * 1024.0 * 1024.0;
/// Below this much dedicated memory the adapter is integrated and lives in shared memory.
const DEDICATED_MIN: usize = 512 * 1024 * 1024;

#[derive(Serialize, Clone, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Gpu {
    name: String,
    /// The busiest engine's load, 0–100.
    usage: f64,
    memory_used_gb: f64,
    memory_total_gb: f64,
    /// Integrated graphics: the memory figures are shared system memory, not the card's own.
    shared: bool,
}

/// An adapter as DXGI describes it, keyed by the LUID the counters use in their instance names.
#[derive(Clone, Debug, PartialEq)]
struct Adapter {
    name: String,
    luid: (u32, u32),
    dedicated: usize,
    shared: usize,
}

fn adapters() -> Vec<Adapter> {
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
        return Vec::new();
    };
    let mut list: Vec<Adapter> = Vec::new();
    for index in 0.. {
        let Ok(adapter) = (unsafe { factory.EnumAdapters1(index) }) else {
            break;
        };
        let Ok(desc) = (unsafe { adapter.GetDesc1() }) else {
            continue;
        };
        // Microsoft Basic Render Driver and other software adapters have no engines to show.
        if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }
        let luid = (desc.AdapterLuid.HighPart as u32, desc.AdapterLuid.LowPart);
        // DXGI lists an adapter once per output it drives; keep the first.
        if list.iter().any(|a| a.luid == luid) {
            continue;
        }
        let end = desc.Description.iter().position(|&c| c == 0).unwrap_or(128);
        list.push(Adapter {
            name: String::from_utf16_lossy(&desc.Description[..end])
                .trim()
                .to_string(),
            luid,
            dedicated: desc.DedicatedVideoMemory,
            shared: desc.SharedSystemMemory,
        });
    }
    list
}

/// "pid_1234_luid_0x00000000_0x00014B2C_phys_0_eng_3_engtype_3D" → (luid, "phys_0_eng_3").
/// Memory instances ("luid_0x00000000_0x00014B2C_phys_0") give (luid, "phys_0").
fn parse_instance(name: &str) -> Option<((u32, u32), String)> {
    let at = name.find("luid_0x")?;
    let mut parts = name[at + "luid_".len()..].splitn(3, '_');
    let high = u32::from_str_radix(parts.next()?.trim_start_matches("0x"), 16).ok()?;
    let low = u32::from_str_radix(parts.next()?.trim_start_matches("0x"), 16).ok()?;
    let rest = parts.next().unwrap_or("");
    let engine = match rest.find("_engtype_") {
        Some(end) => &rest[..end],
        None => rest,
    };
    Some(((high, low), engine.to_string()))
}

/// The busiest engine of one adapter: every process's share of an engine added up, then the
/// highest engine, 0–100.
fn engine_load(samples: &[(String, f64)], luid: (u32, u32)) -> f64 {
    let mut engines: HashMap<String, f64> = HashMap::new();
    for (name, value) in samples {
        if let Some((l, engine)) = parse_instance(name) {
            if l == luid {
                *engines.entry(engine).or_default() += value;
            }
        }
    }
    engines
        .values()
        .copied()
        .fold(0.0, f64::max)
        .clamp(0.0, 100.0)
}

/// Bytes in use on one adapter, over its physical parts.
fn adapter_bytes(samples: &[(String, f64)], luid: (u32, u32)) -> f64 {
    samples
        .iter()
        .filter(|(name, _)| parse_instance(name).is_some_and(|(l, _)| l == luid))
        .map(|(_, v)| v.max(0.0))
        .sum()
}

fn reading(
    adapter: &Adapter,
    engines: &[(String, f64)],
    dedicated: &[(String, f64)],
    shared: &[(String, f64)],
) -> Gpu {
    let integrated = adapter.dedicated < DEDICATED_MIN;
    let (used, total) = if integrated {
        (adapter_bytes(shared, adapter.luid), adapter.shared)
    } else {
        (adapter_bytes(dedicated, adapter.luid), adapter.dedicated)
    };
    Gpu {
        name: adapter.name.clone(),
        usage: engine_load(engines, adapter.luid),
        memory_used_gb: used / GB,
        memory_total_gb: total as f64 / GB,
        shared: integrated,
    }
}

pub struct Reader {
    query: PDH_HQUERY,
    engines: PDH_HCOUNTER,
    dedicated: PDH_HCOUNTER,
    shared: PDH_HCOUNTER,
    adapters: Vec<Adapter>,
}

// The query handle is only used from the system thread that owns the reader.
unsafe impl Send for Reader {}

impl Reader {
    /// None when this PC has no hardware GPU or no GPU counters (some virtual machines).
    pub fn open() -> Option<Reader> {
        let adapters = adapters();
        if adapters.is_empty() {
            return None;
        }
        let mut query = PDH_HQUERY::default();
        if unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) } != 0 {
            return None;
        }
        let add = |path: PCWSTR| {
            let mut counter = PDH_HCOUNTER::default();
            (unsafe { PdhAddEnglishCounterW(query, path, 0, &mut counter) } == 0).then_some(counter)
        };
        let counters = (
            add(w!("\\GPU Engine(*)\\Utilization Percentage")),
            add(w!("\\GPU Adapter Memory(*)\\Dedicated Usage")),
            add(w!("\\GPU Adapter Memory(*)\\Shared Usage")),
        );
        let (Some(engines), Some(dedicated), Some(shared)) = counters else {
            unsafe { PdhCloseQuery(query) };
            return None;
        };
        // Load is a rate: the first collection is the baseline the next one is measured from.
        unsafe { PdhCollectQueryData(query) };
        Some(Reader {
            query,
            engines,
            dedicated,
            shared,
            adapters,
        })
    }

    /// One reading per adapter since the last call (about a second apart).
    pub fn read(&mut self) -> Vec<Gpu> {
        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return Vec::new();
        }
        let engines = values(self.engines);
        let dedicated = values(self.dedicated);
        let shared = values(self.shared);
        self.adapters
            .iter()
            .map(|a| reading(a, &engines, &dedicated, &shared))
            .collect()
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.query) };
    }
}

/// Every instance of a wildcard counter with its value; instances without a valid value (a
/// process that just started) are left out.
fn values(counter: PDH_HCOUNTER) -> Vec<(String, f64)> {
    let format = PDH_FMT(PDH_FMT_DOUBLE.0 | NOCAP100);
    let (mut size, mut count) = (0u32, 0u32);
    let status =
        unsafe { PdhGetFormattedCounterArrayW(counter, format, &mut size, &mut count, None) };
    if status != PDH_MORE_DATA || size == 0 {
        return Vec::new();
    }
    // The items are followed by their names in the same buffer: allocate in whole items, aligned.
    let item = std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>();
    let mut buffer: Vec<PDH_FMT_COUNTERVALUE_ITEM_W> =
        vec![Default::default(); (size as usize).div_ceil(item)];
    let status = unsafe {
        PdhGetFormattedCounterArrayW(
            counter,
            format,
            &mut size,
            &mut count,
            Some(buffer.as_mut_ptr()),
        )
    };
    if status != 0 {
        return Vec::new();
    }
    buffer
        .iter()
        .take(count as usize)
        // PDH_CSTATUS_VALID_DATA (0) or PDH_CSTATUS_NEW_DATA (1).
        .filter(|i| i.FmtValue.CStatus <= 1)
        .filter_map(|i| {
            let name = unsafe { i.szName.to_string() }.ok()?;
            Some((name, unsafe { i.FmtValue.Anonymous.doubleValue }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LUID: (u32, u32) = (0, 0x14B2C);

    fn s(name: &str, v: f64) -> (String, f64) {
        (name.to_string(), v)
    }

    #[test]
    fn reads_engine_and_memory_instances() {
        assert_eq!(
            parse_instance("pid_1234_luid_0x00000000_0x00014B2C_phys_0_eng_3_engtype_3D"),
            Some((LUID, "phys_0_eng_3".to_string()))
        );
        // Counters may come in lower case.
        assert_eq!(
            parse_instance("pid_9_luid_0x00000000_0x00014b2c_phys_1_eng_0_engtype_videodecode"),
            Some((LUID, "phys_1_eng_0".to_string()))
        );
        assert_eq!(
            parse_instance("luid_0x00000000_0x00014B2C_phys_0"),
            Some((LUID, "phys_0".to_string()))
        );
        assert_eq!(parse_instance("_Total"), None);
    }

    #[test]
    fn load_is_the_busiest_engine_with_all_its_processes() {
        let samples = [
            // 3D engine: two processes, 30 + 25.
            s(
                "pid_1_luid_0x00000000_0x00014B2C_phys_0_eng_0_engtype_3D",
                30.0,
            ),
            s(
                "pid_2_luid_0x00000000_0x00014B2C_phys_0_eng_0_engtype_3D",
                25.0,
            ),
            // Video decode: 40 on its own.
            s(
                "pid_3_luid_0x00000000_0x00014B2C_phys_0_eng_4_engtype_VideoDecode",
                40.0,
            ),
            // Another adapter is not counted.
            s(
                "pid_1_luid_0x00000000_0x00099999_phys_0_eng_0_engtype_3D",
                90.0,
            ),
        ];
        assert_eq!(engine_load(&samples, LUID), 55.0);
        assert_eq!(engine_load(&[], LUID), 0.0);
    }

    #[test]
    fn load_stays_within_100() {
        let samples = [
            s(
                "pid_1_luid_0x00000000_0x00014B2C_phys_0_eng_0_engtype_3D",
                80.0,
            ),
            s(
                "pid_2_luid_0x00000000_0x00014B2C_phys_0_eng_0_engtype_3D",
                45.0,
            ),
        ];
        assert_eq!(engine_load(&samples, LUID), 100.0);
    }

    #[test]
    fn integrated_graphics_show_shared_memory() {
        let gb = 1024 * 1024 * 1024;
        let igpu = Adapter {
            name: "Intel(R) UHD Graphics".into(),
            luid: LUID,
            dedicated: 128 << 20,
            shared: 8 * gb,
        };
        let dedicated = [s("luid_0x00000000_0x00014B2C_phys_0", 0.0)];
        let shared = [s("luid_0x00000000_0x00014B2C_phys_0", 2.0 * GB)];
        let g = reading(&igpu, &[], &dedicated, &shared);
        assert!(g.shared);
        assert_eq!(g.memory_used_gb, 2.0);
        assert_eq!(g.memory_total_gb, 8.0);

        let card = Adapter {
            name: "Graphics card".into(),
            luid: LUID,
            dedicated: 8 * gb,
            shared: 8 * gb,
        };
        let dedicated = [s("luid_0x00000000_0x00014B2C_phys_0", 3.0 * GB)];
        let g = reading(&card, &[], &dedicated, &shared);
        assert!(!g.shared);
        assert_eq!(g.memory_used_gb, 3.0);
        assert_eq!(g.memory_total_gb, 8.0);
    }
}
