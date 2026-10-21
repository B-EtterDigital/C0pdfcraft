#[path = "../src/c0_web.rs"]
mod browser;

#[test]
fn exported_files_use_their_own_mime_type() {
    for (name, mime) in [
        ("report.PDF", "application/pdf"),
        ("page.png", "image/png"),
        ("data.csv", "text/csv;charset=utf-8"),
        ("reply.xfdf", "application/xml"),
        ("archive.zip", "application/zip"),
        ("unknown", "application/octet-stream"),
    ] {
        assert_eq!(browser::mime(name), mime);
    }
}

#[test]
fn browser_helpers_keep_native_clock_and_session_defaults() {
    assert!(browser::session().now_secs() > 1_700_000_000);
    assert!(browser::system_time().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() > 1_700_000_000);
}
