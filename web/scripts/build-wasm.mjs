// Builds crates/hh-wasm into web/src/wasm/ (git-ignored), so the page always runs the
// engine from this checkout. Run by `npm run dev`, `npm run game` and `npm run build`,
// and by the dev server's watcher when crates/, content/ or scenarios/ change.
//   HH_WASM_PROFILE=dev   faster unoptimized build (default: release)
//   HH_WASM_FORCE=1       build even when nothing the wasm is built from has changed
//
// A build is skipped (in well under a second) when web/src/wasm/ already holds a build of
// the same inputs: every file under crates/hh-core and crates/hh-wasm (but target/ and
// pkg/), content/ and scenarios/, the workspace Cargo.toml and Cargo.lock, this script
// and the profile,
// hashed by content into web/src/wasm/.inputs. A build that runs costs most of a warm
// start even when the Rust is unchanged (wasm-opt reruns every time).

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const repo = resolve(web, '..');
const crate = join(repo, 'crates', 'hh-wasm');
const outDir = join(web, 'src', 'wasm');
const stamp = join(outDir, '.inputs');
const profile = process.env.HH_WASM_PROFILE === 'dev' ? '--dev' : '--release';

/** Every input file under `dir`, sorted, skipping build output folders. */
function files(dir) {
  if (!existsSync(dir)) return [];
  const out = [];
  for (const e of readdirSync(dir, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : 1))) {
    const p = join(dir, e.name);
    if (e.isDirectory()) {
      if (e.name !== 'target' && e.name !== 'pkg' && e.name !== 'node_modules') out.push(...files(p));
    } else if (e.isFile()) out.push(p);
  }
  return out;
}

function inputsHash() {
  const h = createHash('sha256');
  h.update(`profile ${profile}\n`);
  const list = [
    // The crates the wasm is built from (hh-wasm, hh-core, and hh-sim for the debug
    // autopilot's sim players), and the milestones the autopilot embeds.
    ...[join('crates', 'hh-core'), join('crates', 'hh-sim'), join('crates', 'hh-wasm'), 'content', 'scenarios'].flatMap((d) =>
      files(join(repo, d)),
    ),
    join(repo, 'configs', 'sim', 'milestones.json'),
    join(repo, 'Cargo.toml'),
    join(repo, 'Cargo.lock'),
    fileURLToPath(import.meta.url),
  ];
  for (const f of list) {
    if (!existsSync(f)) continue;
    h.update(`${relative(repo, f).replaceAll('\\', '/')}\n`);
    h.update(readFileSync(f));
  }
  return h.digest('hex');
}

const started = Date.now();
const hash = inputsHash();
const built = ['hh_wasm.js', 'hh_wasm_bg.wasm'].every((f) => existsSync(join(outDir, f)));
if (!process.env.HH_WASM_FORCE && built && existsSync(stamp) && readFileSync(stamp, 'utf8').trim() === hash) {
  console.log(`web/src/wasm is up to date (${profile.slice(2)}; checked in ${((Date.now() - started) / 1000).toFixed(1)} s)`);
  process.exit(0);
}

// A build that fails part way must not leave a stamp vouching for the old files.
rmSync(stamp, { force: true });
const build = spawnSync(
  'wasm-pack',
  ['build', crate, '--target', 'web', profile, '--out-dir', outDir, '--out-name', 'hh_wasm', '--no-pack'],
  { stdio: 'inherit', shell: process.platform === 'win32' },
);
if (build.error) {
  console.error(`Could not run wasm-pack (cargo install wasm-pack --locked): ${build.error.message}`);
  process.exit(1);
}
if (build.status !== 0) process.exit(build.status ?? 1);
rmSync(join(outDir, '.gitignore'), { force: true });
writeFileSync(stamp, `${hash}\n`, { encoding: 'utf-8' });
console.log(`web/src/wasm built (${profile.slice(2)}) in ${((Date.now() - started) / 1000).toFixed(1)} s`);
