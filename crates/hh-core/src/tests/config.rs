use serde_json::{json, Value};

use super::{baseline, BASELINE};
use crate::{Config, Rules};

fn edited(f: impl FnOnce(&mut Value)) -> Result<Config, String> {
    let mut v: Value = serde_json::from_str(BASELINE).unwrap();
    f(&mut v);
    Config::from_value(&v)
}

fn params<'a>(v: &'a mut Value, module: &str) -> &'a mut serde_json::Map<String, Value> {
    v["modules"][module]["params"].as_object_mut().unwrap()
}

fn economy(v: &mut Value) -> &mut serde_json::Map<String, Value> {
    params(v, "economy")
}

#[test]
fn baseline_loads_and_round_trips() {
    let c = baseline();
    let again = Config::from_json(&c.to_json()).unwrap();
    assert_eq!(again, c);
    assert_eq!(again.hash(), c.hash());
    Rules::new(&c).unwrap();
}

#[test]
fn a_missing_param_is_named() {
    let e = edited(|v| {
        economy(v).remove("land_base_cost");
    })
    .unwrap_err();
    assert!(e.contains("modules.economy.params.land_base_cost: missing"), "{e}");
}

#[test]
fn every_missing_param_is_named() {
    let e = edited(|v| {
        economy(v).remove("land_base_cost");
        params(v, "boredom").remove("phase_rates");
    })
    .unwrap_err();
    assert!(e.contains("land_base_cost: missing") && e.contains("phase_rates: missing"), "{e}");
}

#[test]
fn an_unknown_param_is_named() {
    let e = edited(|v| {
        economy(v).insert("land_base_costs".into(), json!("1"));
    })
    .unwrap_err();
    assert!(e.contains("modules.economy.params.land_base_costs: unknown parameter"), "{e}");
}

#[test]
fn unknown_keys_modules_and_variants_are_named() {
    let e = edited(|v| {
        v["extra"] = json!(1);
    })
    .unwrap_err();
    assert!(e.contains("extra: unknown key"), "{e}");
    let e = edited(|v| {
        v["modules"]["fusion"] = json!({});
    })
    .unwrap_err();
    assert!(e.contains("modules.fusion: unknown module"), "{e}");
    let e = edited(|v| {
        v["modules"]["dream"]["variant"] = json!("echo");
    })
    .unwrap_err();
    assert!(e.contains("unknown variant \"echo\""), "{e}");
    let e = edited(|v| {
        v.as_object_mut().unwrap().remove("name");
    })
    .unwrap_err();
    assert!(e.contains("name: missing"), "{e}");
}

#[test]
fn a_missing_module_is_refused_by_the_rules() {
    let c = edited(|v| {
        v["modules"].as_object_mut().unwrap().remove("dream");
    })
    .unwrap();
    let e = Rules::new(&c).err().unwrap();
    assert!(e.contains("missing module(s) dream"), "{e}");
}

#[test]
fn wrong_kinds_and_ranges_are_named() {
    let e = edited(|v| {
        economy(v).insert("land_base_cost".into(), json!(100));
    })
    .unwrap_err();
    assert!(e.contains("land_base_cost: expected a decimal string"), "{e}");
    let e = edited(|v| {
        economy(v).insert("land_base_cost".into(), json!("1.0000001"));
    })
    .unwrap_err();
    assert!(e.contains("land_base_cost: \"1.0000001\" is not a decimal"), "{e}");
    let e = edited(|v| {
        economy(v).insert("land_cost_scaling".into(), json!("0.5"));
    })
    .unwrap_err();
    assert!(e.contains("land_cost_scaling: out of range"), "{e}");
}

#[test]
fn lists_and_maps_are_checked_element_by_element() {
    let e = edited(|v| {
        params(v, "boredom").insert("phase_rates".into(), json!(["0.1", 2]));
    })
    .unwrap_err();
    assert!(e.contains("phase_rates: [1]: expected a decimal string"), "{e}");
    let e = edited(|v| {
        economy(v).insert("starting_buildings".into(), json!({"solar_panel": "1"}));
    })
    .unwrap_err();
    assert!(e.contains("starting_buildings: solar_panel: expected an integer"), "{e}");
    let e = edited(|v| {
        params(v, "boredom").insert("phase_rates".into(), json!(["0.1", "-1"]));
    })
    .unwrap_err();
    assert!(e.contains("phase_rates: out of range"), "{e}");
    // Content ids in a map are checked when the rules are built.
    let c = edited(|v| {
        economy(v).insert("starting_buildings".into(), json!({"fusion": 1}));
    })
    .unwrap();
    let e = Rules::new(&c).err().unwrap();
    assert!(e.contains("starting_buildings.fusion: unknown building"), "{e}");
}

#[test]
fn the_hash_ignores_name_and_spelling() {
    let a = baseline();
    let b = edited(|v| {
        v["name"] = json!("other");
        economy(v).insert("land_base_cost".into(), json!("200.000"));
    })
    .unwrap();
    assert_eq!(a.hash(), b.hash());
    let c = edited(|v| {
        economy(v).insert("land_base_cost".into(), json!("201"));
    })
    .unwrap();
    assert_ne!(a.hash(), c.hash());
}
