//! Platform clock, separated so a tiny test-only wasm probe can exercise the real helper.

#[cfg(target_arch = "wasm32")]
pub(super) fn epoch_seconds() -> i64 {
    (js_sys::Date::now() / 1000.0) as i64
}

#[cfg(target_arch = "wasm32")]
pub(super) fn utc_offset() -> i64 {
    (-js_sys::Date::new_0().get_timezone_offset() * 60.0) as i64
}

pub(crate) fn system_time() -> std::time::SystemTime {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
    }
    #[cfg(target_arch = "wasm32")]
    {
        let seconds = u64::try_from(epoch_seconds()).unwrap_or(0);
        std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(seconds)).unwrap_or(std::time::UNIX_EPOCH)
    }
}
