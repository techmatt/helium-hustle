//! `hh-sim params [--config F] [--write]`: each module's parameter table, generated
//! from `hh_core::modules::REGISTRY` (name, range, doc) and a config's values, so the
//! tables in `docs/rules/<module>.md` have one home. A page holds its table between
//! `<!-- params:<module> -->` and `<!-- /params -->`; `--write` rewrites those blocks
//! (with the baseline config), and a test fails when one differs.

use hh_core::config::Param;
use hh_core::modules::{ModuleDef, REGISTRY};
use hh_core::{Config, Fx};
use std::path::Path;

use crate::{usage, Fail, BASELINE};

/// The command that regenerates the docs tables, named by the test that checks them.
pub const REGENERATE: &str = "cargo run -q --release -p hh-sim -- params --write";

const CLOSE: &str = "<!-- /params -->";

fn open(id: &str) -> String {
    format!("<!-- params:{id} -->")
}

/// An `Fx` as the configs write it: no trailing zeros (`1.5`, `100`).
fn fx(v: &Fx) -> String {
    let s = v.to_string();
    let s = s.trim_end_matches('0');
    s.strip_suffix('.').unwrap_or(s).to_string()
}

fn value(p: &Param) -> String {
    fn join(items: impl Iterator<Item = String>) -> String {
        items.collect::<Vec<_>>().join(", ")
    }
    match p {
        Param::Int(v) => v.to_string(),
        Param::Fx(v) => fx(v),
        Param::IntList(l) => join(l.iter().map(i64::to_string)),
        Param::FxList(l) => join(l.iter().map(fx)),
        Param::IntMap(m) => join(m.iter().map(|(k, v)| format!("{k} {v}"))),
        Param::FxMap(m) => join(m.iter().map(|(k, v)| format!("{k} {}", fx(v)))),
    }
}

/// A `ParamDef.doc` as a table cell: a capital and a period, snake_case names in code
/// spans.
fn meaning(doc: &str) -> String {
    let words: Vec<String> = doc
        .split(' ')
        .map(|w| {
            let core = w.trim_end_matches([',', ')', ';']);
            if core.contains('_') && core.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                format!("`{core}`{}", &w[core.len()..])
            } else {
                w.to_string()
            }
        })
        .collect();
    let s = words.join(" ");
    let mut c = s.chars();
    let first = c.next().map(|f| f.to_uppercase().collect::<String>()).unwrap_or_default();
    format!("{first}{}.", c.as_str())
}

/// One module's block (without its markers): a table per variant, baseline values
/// from `config` for the variant it plays.
pub fn table(def: &ModuleDef, config: &Config) -> String {
    let played = config.module(def.id);
    let mut out = String::new();
    for v in def.variants {
        if def.variants.len() > 1 {
            out.push_str(&format!("Variant `{}`:\n\n", v.name));
        }
        if v.params.is_empty() {
            out.push_str(&format!("The `{}` variant has no params (`\"params\": {{}}`).\n", v.name));
            continue;
        }
        out.push_str("| Param | Baseline | Range | Meaning |\n|---|---|---|---|\n");
        for p in v.params {
            let base = played
                .filter(|m| m.variant == v.name)
                .and_then(|m| m.params.get(p.name))
                .map(value)
                .unwrap_or_default();
            let range = p.range.map(|(lo, hi)| format!("{lo} to {hi}")).unwrap_or_default();
            out.push_str(&format!("| `{}` | {base} | {range} | {} |\n", p.name, meaning(p.doc)));
        }
    }
    out
}

/// Every module's block with its markers, in registry order.
pub fn all(config: &Config) -> String {
    REGISTRY
        .iter()
        .map(|d| format!("{}\n{}{CLOSE}\n", open(d.id), table(d, config)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `page` with its `id` block replaced by `block`; `Err` when the markers are missing.
pub fn splice(page: &str, id: &str, block: &str) -> Result<String, String> {
    let open = open(id);
    let start = page.find(&open).ok_or_else(|| format!("no {open} marker"))? + open.len();
    let end = start + page[start..].find(CLOSE).ok_or_else(|| format!("no {CLOSE} after {open}"))?;
    Ok(format!("{}\n{block}{}", &page[..start], &page[end..]))
}

/// The pages under `root/docs/rules/` whose table differs from the generated one
/// (each with why); with `write`, rewrites them instead.
pub fn sync(root: &Path, config: &Config, write: bool) -> Result<Vec<String>, String> {
    let mut stale = Vec::new();
    for d in REGISTRY {
        let path = root.join("docs").join("rules").join(format!("{}.md", d.id));
        let page = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let want = splice(&page, d.id, &table(d, config)).map_err(|e| format!("{}: {e}", path.display()))?;
        if want != page {
            if write {
                std::fs::write(&path, want).map_err(|e| format!("{}: {e}", path.display()))?;
            }
            stale.push(format!("docs/rules/{}.md", d.id));
        }
    }
    Ok(stale)
}

pub fn run(args: &[String]) -> Result<String, Fail> {
    let mut config = BASELINE.to_string();
    let mut write = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--config" => config = it.next().cloned().ok_or_else(|| Fail::Usage("--config needs a value".into()))?,
            "--write" => write = true,
            other => return usage(format!("params: unexpected argument {other:?}")),
        }
    }
    if write && config != BASELINE {
        return usage("params: --write writes the baseline's values; drop --config");
    }
    let c = crate::load_config(&config)?;
    if !write {
        return Ok(all(&c));
    }
    let changed = sync(Path::new("."), &c, true).map_err(Fail::Usage)?;
    Ok(if changed.is_empty() {
        "docs/rules param tables already match\n".into()
    } else {
        format!("rewrote {}\n", changed.join(", "))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
    }

    #[test]
    fn docs_rules_param_tables_match_the_registry() {
        let root = root();
        let json = std::fs::read_to_string(root.join(BASELINE)).expect("baseline config");
        let c = Config::from_json(&json).expect("baseline loads");
        let stale = sync(&root, &c, false).expect("every module page has its params markers");
        assert!(
            stale.is_empty(),
            "param tables differ from REGISTRY in {}: run `{REGENERATE}`",
            stale.join(", ")
        );
    }

    #[test]
    fn meaning_capitalizes_and_marks_code_names() {
        assert_eq!(meaning("demand ceiling before demand_ceiling modifiers"), "Demand ceiling before `demand_ceiling` modifiers.");
        assert_eq!(meaning("x (a, promote_effectiveness_mult)"), "X (a, `promote_effectiveness_mult`).");
    }

    #[test]
    fn splice_replaces_only_the_block() {
        let page = "a\n<!-- params:m -->\nold\n<!-- /params -->\nb\n";
        assert_eq!(splice(page, "m", "new\n").unwrap(), "a\n<!-- params:m -->\nnew\n<!-- /params -->\nb\n");
        assert!(splice(page, "n", "x").is_err());
    }
}
