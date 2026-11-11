#[path = "../src/c0_web.rs"]
mod browser;

// Stands in for upstream's `download_mime_type`, which the real build passes in; the table itself is
// tested against the crate in tests/it/download_mime.rs.
fn upstream(_name: &str) -> &'static str {
    "upstream/type"
}

#[test]
fn zip_downloads_as_zip_and_other_names_follow_upstream() {
    assert_eq!(browser::mime("archive.ZIP", upstream), "application/zip");
    assert_eq!(browser::mime("report.PDF", upstream), "upstream/type");
    assert_eq!(browser::mime("unknown", upstream), "upstream/type");
}

#[test]
fn browser_helpers_keep_native_clock_and_session_defaults() {
    assert!(browser::session().now_secs() > 1_700_000_000);
    assert!(browser::system_time().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() > 1_700_000_000);
}
