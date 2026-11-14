// Builds crates/hh-wasm into web/src/wasm/ (git-ignored), so the page always runs the
// engine from this checkout. Run by `npm run dev` and `npm run build`, and by the dev
// server's watcher when crates/, content/ or scenarios/ change.
//   HH_WASM_PROFILE=dev   faster unoptimized build (default: release)

import { spawnSync } from 'node:child_process';
import { rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const crate = join(web, '..', 'crates', 'hh-wasm');
const outDir = join(web, 'src', 'wasm');
const profile = process.env.HH_WASM_PROFILE === 'dev' ? '--dev' : '--release';

const started = Date.now();
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
console.log(`web/src/wasm built (${profile.slice(2)}) in ${((Date.now() - started) / 1000).toFixed(1)} s`);
