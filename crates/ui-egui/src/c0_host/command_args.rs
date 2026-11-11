//! The `params` of `ui.command`. A registry command runs with parameters only when it is listed here.
//! Any other command refuses non-empty `params`, and a listed command refuses a value it cannot use.
//! A refusal never falls back to the command's default, so the caller learns that nothing ran.

use crate::PdfCraftApp;
use pdfcraft_engine::Edit;
use serde_json::{Map, Value, json};

/// Run registry command `id` for a `ui.command` request that carried `params`. The caller has
/// already checked that the command is enabled. Absent, `null` and `{}` all mean "no parameters".
pub(crate) fn run(app: &mut PdfCraftApp, id: &str, params: &Value) -> Result<Value, String> {
    let given = given(id, params)?;
    match id {
        "page.rotate" => {
            let degrees = rotate_degrees(given)?;
            let pages = app.active.and_then(|i| app.views.get(i)).map(|v| v.target_pages()).unwrap_or_default();
            // The same targets as the menu's command; only the angle comes from the request.
            let applied = pdfcraft_engine::guard(|| app.apply_edit(Edit::RotatePages { pages, degrees }))
                .map_err(|m| format!("{id}: an internal error stopped it ({m}). Your documents are unchanged."))?;
            if !applied {
                return Err(format!("{id}: no pages were rotated (no document is open, or the edit failed)"));
            }
            Ok(json!({ "ran": id, "degrees": degrees }))
        }
        _ => {
            if let Some(map) = given {
                let names: Vec<&str> = map.keys().map(String::as_str).collect();
                return Err(format!("{id} takes no parameters (got {}); send it without params", names.join(", ")));
            }
            app.execute(id);
            Ok(json!({ "ran": id }))
        }
    }
}

/// The parameters as an object, or `None` when the request sent none.
fn given<'a>(id: &str, params: &'a Value) -> Result<Option<&'a Map<String, Value>>, String> {
    match params {
        Value::Null => Ok(None),
        Value::Object(map) if map.is_empty() => Ok(None),
        Value::Object(map) => Ok(Some(map)),
        _ => Err(format!("{id}: params must be an object")),
    }
}

/// `page.rotate`'s `degrees`: a whole multiple of 90, positive is clockwise, as `page_rotate` in the
/// automation tools takes it. Without parameters it is the menu's quarter turn clockwise.
fn rotate_degrees(given: Option<&Map<String, Value>>) -> Result<i64, String> {
    let Some(map) = given else { return Ok(90) };
    if let Some(key) = map.keys().find(|k| k.as_str() != "degrees") {
        return Err(format!("page.rotate takes only degrees (got {key:?})"));
    }
    let value = map.get("degrees").ok_or("page.rotate takes degrees (a multiple of 90)")?;
    match value.as_i64() {
        Some(d) if d % 90 == 0 => Ok(d),
        _ => Err(format!("page.rotate: degrees must be a whole multiple of 90, such as 90, 180, 270 or -90; got {value}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_null_and_empty_params_mean_none() {
        assert!(given("edit.undo", &Value::Null).unwrap().is_none());
        assert!(given("edit.undo", &json!({})).unwrap().is_none());
        assert!(given("edit.undo", &json!({"degrees": 90})).unwrap().is_some());
        for bad in [json!(3), json!("x"), json!([]), json!(true)] {
            assert!(given("edit.undo", &bad).unwrap_err().contains("must be an object"));
        }
    }

    #[test]
    fn rotate_takes_whole_quarter_turns_and_nothing_else() {
        assert_eq!(rotate_degrees(None), Ok(90));
        for (params, want) in
            [(json!({"degrees": 90}), 90), (json!({"degrees": 180}), 180), (json!({"degrees": -90}), -90), (json!({"degrees": 270}), 270)]
        {
            assert_eq!(rotate_degrees(params.as_object()), Ok(want));
        }
        for bad in [
            json!({"degrees": 45}),
            json!({"degrees": "180"}),
            json!({"degrees": 90.0}),
            json!({"degrees": null}),
            json!({"angle": 90}),
            json!({"degrees": 90, "pages": [1]}),
        ] {
            assert!(rotate_degrees(bad.as_object()).is_err(), "{bad} must be refused, not defaulted");
        }
    }
}
