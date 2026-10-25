use super::*;

#[test]
fn native_feature_parses_and_bounds_requests() {
    assert_eq!(parse(r#"{"method":"c0.inspect"}"#).unwrap().1, json!({}));
    assert_eq!(parse(r#"{"method":"c0.inspect","params":null}"#).unwrap().1, json!({}));
    for text in ["", "[]", "{}", "oops", r#"{"method":0}"#, r#"{"method":"c0.inspect","params":3}"#] {
        assert!(parse(text).is_err());
    }
    let large = json!({"method": "ui.type", "params": {"text": "x".repeat(MAX_REQUEST_BYTES)}}).to_string();
    assert!(parse(&large).is_err());
    assert!(parse(&large.replace("ui.type", "c0.openFile")).is_ok());
    assert!(parse(r#"{"method":"ui.drag","params":{"steps":33}}"#).is_err());
}

#[test]
fn envelopes_and_read_policy_compile_on_native() {
    assert_eq!(serde_json::from_str::<Value>(&encode(Ok(json!({"a":1})))).unwrap(), json!({"ok":true,"result":{"a":1}}));
    assert_eq!(serde_json::from_str::<Value>(&encode(Err("bad".into()))).unwrap(), json!({"ok":false,"error":"bad"}));
    assert!(reads_only("c0.inspect"));
    assert!(reads_only("c0.documentBytes"));
    assert!(reads_only("c0.selection"));
    assert!(reads_only("ui.render"));
    for method in ["c0.set", "c0.place", "c0.goTo", "c0.newDocument", "c0.openFile", "c0.cancelFileDialog"] {
        assert!(!reads_only(method));
    }
}
