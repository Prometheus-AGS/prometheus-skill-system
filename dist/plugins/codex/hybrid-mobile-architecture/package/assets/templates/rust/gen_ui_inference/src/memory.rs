// TJ-ARCH-MOB-001 compliant
//! Device-memory preflight shared by the mobile inference engines.
//!
//! iOS terminates a process that exceeds its jetsam allowance rather than
//! failing an allocation, so an over-budget model load cannot be caught as an
//! `Err` after the fact — it has to be refused (or downgraded to a smaller
//! model) before the load starts. Engines differ in how much memory a model
//! really occupies beyond its artifact bytes (mmap'd GGUF vs MLX unified-memory
//! buffers), so callers pass their own peak-bytes estimate and fallback list;
//! this module owns only the device-budget arithmetic.

use gen_ui_types::inference::LocalModelSpec;

/// Whether a model whose estimated peak residency is `peak_bytes` exceeds this
/// device's safe budget. `false` on platforms with no usable budget signal
/// (desktop, where the OS pages) — never produces an error, so the caller can
/// choose a smaller model instead of failing outright.
pub(crate) fn preflight_rejects(peak_bytes: u64) -> bool {
    device_memory_budget_bytes().is_some_and(|budget| peak_bytes > budget)
}

/// The next-smaller cataloged model that fits this device, if one exists.
///
/// `fallbacks` is ordered largest-to-smallest; `peak_bytes_for_id` maps a
/// catalog id to that engine's estimated peak residency (returning `None` for
/// unknown ids — an absolute path is the caller's own choice and is never
/// downgraded). Returns `None` when the requested model already fits (no
/// downgrade needed) or when no cataloged alternative is small enough.
/// Self-guarding so a caller cannot accidentally downgrade a model that was
/// fine as-is.
pub(crate) fn downgrade_for_memory(
    spec: &LocalModelSpec,
    fallbacks: &[&str],
    peak_bytes_for_id: impl Fn(&str) -> Option<u64>,
) -> Option<LocalModelSpec> {
    let requested_peak = peak_bytes_for_id(&spec.model)?;
    if !preflight_rejects(requested_peak) {
        return None;
    }
    fallbacks
        .iter()
        .filter(|candidate| **candidate != spec.model)
        .find_map(|candidate| {
            let peak = peak_bytes_for_id(candidate)?;
            (!preflight_rejects(peak)).then(|| LocalModelSpec {
                model: (*candidate).to_owned(),
                context_len: spec.context_len,
            })
        })
}

/// The memory a local model may safely occupy on this device, or `None` when
/// the platform gives no usable signal (desktop, where the OS pages).
///
/// The fraction is intentionally conservative: the OS, other apps, and this
/// app's non-model allocations all draw on the same physical memory.
/// `__ENV_PREFIX___DEVICE_MEMORY_BUDGET_BYTES` overrides the computed budget (tests,
/// device tuning).
pub(crate) fn device_memory_budget_bytes() -> Option<u64> {
    if let Some(override_bytes) = std::env::var("__ENV_PREFIX___DEVICE_MEMORY_BUDGET_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
    {
        return Some(override_bytes);
    }
    #[cfg(target_os = "ios")]
    {
        // Share of physical RAM an app can hold before jetsam intervenes, even
        // with com.apple.developer.kernel.increased-memory-limit granted.
        const IOS_JETSAM_SAFE_FRACTION: f64 = 0.55;
        let physical = physical_memory_bytes()?;
        return Some((physical as f64 * IOS_JETSAM_SAFE_FRACTION) as u64);
    }
    #[cfg(not(target_os = "ios"))]
    {
        None
    }
}

#[cfg(target_os = "ios")]
fn physical_memory_bytes() -> Option<u64> {
    // sysctl hw.memsize is the documented total-physical-memory query on Darwin.
    let mut size: u64 = 0;
    let mut length = std::mem::size_of::<u64>();
    let name = c"hw.memsize";
    // SAFETY: `name` is a valid NUL-terminated C string; `size` is a live u64
    // and `length` accurately describes its size, which is what sysctlbyname
    // requires for the output buffer. The return value is checked before `size`
    // is read.
    let result = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            std::ptr::addr_of_mut!(size).cast(),
            std::ptr::addr_of_mut!(length),
            std::ptr::null_mut(),
            0,
        )
    };
    (result == 0 && size > 0).then_some(size)
}
