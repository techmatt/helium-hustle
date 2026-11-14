use serde_json::{json, Value};

use super::{baseline, BASELINE};
use crate::Config;

fn edited(f: impl FnOnce(&mut Value)) -> Result<Config, String> {
    let mut v: Value = serde_json::from_str(BASELINE).unwrap();
    f(&mut v);
    Config::from_value(&v)
}

fn params(v: &mut Value) -> &mut serde_json::Map<String, Value> {
    v["modules"]["placeholder"]["params"].as_object_mut().unwrap()
}

#[test]
fn baseline_loads_and_round_trips() {
    let c = baseline();
    let again = Config::from_json(&c.to_json()).unwrap();
    assert_eq!(again, c);
    assert_eq!(again.hash(), c.hash());
}

#[test]
fn a_missing_param_is_named() {
    let e = edited(|v| {
        params(v).remove("energy_cap");
    })
    .unwrap_err();
    assert!(e.contains("modules.placeholder.params.energy_cap: missing"), "{e}");
}

#[test]
fn every_missing_param_is_named() {
    let e = edited(|v| {
        params(v).remove("energy_cap");
        params(v).remove("cost_growth");
    })
    .unwrap_err();
    assert!(e.contains("energy_cap: missing") && e.contains("cost_growth: missing"), "{e}");
}

#[test]
fn an_unknown_param_is_named() {
    let e = edited(|v| {
        params(v).insert("energy_capp".into(), json!("1"));
    })
    .unwrap_err();
    assert!(e.contains("modules.placeholder.params.energy_capp: unknown parameter"), "{e}");
}

#[test]
fn unknown_keys_modules_and_variants_are_named() {
    let e = edited(|v| {
        v["extra"] = json!(1);
    })
    .unwrap_err();
    assert!(e.contains("extra: unknown key"), "{e}");
    let e = edited(|v| {
        v["modules"]["shipments"] = json!({});
    })
    .unwrap_err();
    assert!(e.contains("modules.shipments: unknown module"), "{e}");
    let e = edited(|v| {
        v["modules"]["placeholder"]["variant"] = json!("fancy");
    })
    .unwrap_err();
    assert!(e.contains("unknown variant \"fancy\""), "{e}");
    let e = edited(|v| {
        v.as_object_mut().unwrap().remove("name");
    })
    .unwrap_err();
    assert!(e.contains("name: missing"), "{e}");
}

#[test]
fn wrong_kinds_and_ranges_are_named() {
    let e = edited(|v| {
        params(v).insert("energy_cap".into(), json!(100));
    })
    .unwrap_err();
    assert!(e.contains("energy_cap: expected a decimal string"), "{e}");
    let e = edited(|v| {
        params(v).insert("energy_cap".into(), json!("1.0000001"));
    })
    .unwrap_err();
    assert!(e.contains("energy_cap: \"1.0000001\" is not a decimal"), "{e}");
    let e = edited(|v| {
        params(v).insert("sunlight_spread".into(), json!("1.5"));
    })
    .unwrap_err();
    assert!(e.contains("sunlight_spread: out of range"), "{e}");
}

#[test]
fn the_hash_ignores_name_and_spelling() {
    let a = baseline();
    let b = edited(|v| {
        v["name"] = json!("other");
        params(v).insert("energy_cap".into(), json!("100.000"));
    })
    .unwrap();
    assert_eq!(a.hash(), b.hash());
    let c = edited(|v| {
        params(v).insert("energy_cap".into(), json!("101"));
    })
    .unwrap();
    assert_ne!(a.hash(), c.hash());
}
