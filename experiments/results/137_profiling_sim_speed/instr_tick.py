import re, pathlib
p = pathlib.Path(r"C:\Code\helium-hustle-worktrees\helium-hustle\137_profiling_sim_speed\crates\hh-core\src\rules.rs")
s = p.read_text()
a = s.index("    fn tick(&self, g: &mut GameState) {")
b = s.index("    /// Applies a player action between ticks.")
body = s[a:b]
names = []
out = []
for ln in body.split("\n"):
    m = re.match(r"^(\s+)((?:let \w+ = )?)((?:\w+::)+\w+\(.*\));$", ln)
    if m:
        names.append(m.group(3).split("(")[0])
        i = 40 + len(names) - 1
        ln = f"{m.group(1)}{m.group(2)}{{ let t = std::time::Instant::now(); let r = {m.group(3)}; crate::view::VT[{i}].fetch_add(t.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed); r }};"
    out.append(ln)
s = s[:a] + "\n".join(out) + s[b:]
s += "\npub static TICK_NAMES: &[&str] = &[" + ", ".join(f'"{n}"' for n in names) + "];\n"
p.write_bytes(s.encode())
print(names)
