import re, pathlib
p = pathlib.Path(r"C:\Code\helium-hustle-worktrees\helium-hustle\137_profiling_sim_speed\crates\hh-core\src\view.rs")
s = p.read_text()
start = s.index("pub fn build(r: &Rules, g: &GameState) -> View {")
body = s[start:]
names = []
def wrap(name, expr):
    names.append(name)
    return f"vt!({len(names)-1}, {expr})"
lines = body.split("\n")
out = []
for ln in lines:
    m = re.match(r"^(    let (\w+) = )(.*);$", ln)
    if m and m.group(2) not in ("c", "same_run"):
        ln = m.group(1) + wrap(m.group(2), m.group(3)) + ";"
    m = re.match(r"^(        (\w+): )(\w+::\w+(?:::\w+)?\(.*\)|self::projects::\w+\(.*\)|breakdown\(g\)|names\(\)|axes\(r, g\)),$", ln)
    if m:
        ln = m.group(1) + wrap(m.group(2), m.group(3)) + ","
    out.append(ln)
    if ln.startswith("}"):
        break
new_body = "\n".join(out)
rest = body[len("\n".join(lines[:len(out)])):]
macro = ("pub static VT: [std::sync::atomic::AtomicU64; 64] = [const { std::sync::atomic::AtomicU64::new(0) }; 64];\n"
         "pub static VT_NAMES: &[&str] = &[" + ", ".join(f'"{n}"' for n in names) + "];\n"
         "macro_rules! vt { ($i:expr, $e:expr) => {{ let t = std::time::Instant::now(); let r = $e; VT[$i].fetch_add(t.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed); r }} }\n")
s2 = s[:start] + macro + new_body + rest
p.write_text(s2)
print(len(names), names)
