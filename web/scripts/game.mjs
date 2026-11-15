// `npm run game -- [vite args]`: the dev server, started as fast as the checkout allows
// (conductor's Run game target). The same server as `npm run dev`, with two steps skipped
// when they would do nothing:
// - `npm install`, when package.json and package-lock.json are byte for byte what the
//   last install here saw (a hash in node_modules/.hh-install);
// - the wasm build, when its inputs are unchanged (scripts/build-wasm.mjs).
// Needs nothing installed to start: it runs `npm install` itself on a fresh checkout.

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const stamp = join(web, 'node_modules', '.hh-install');
const vite = join(web, 'node_modules', 'vite', 'bin', 'vite.js');

const run = (cmd, args, shell = false) => {
  const r = spawnSync(cmd, args, { cwd: web, stdio: 'inherit', shell });
  if (r.error) {
    console.error(`Could not run ${cmd}: ${r.error.message}`);
    process.exit(1);
  }
  if (r.status !== 0) process.exit(r.status ?? 1);
};

const lockHash = () => {
  const h = createHash('sha256');
  for (const f of ['package.json', 'package-lock.json']) {
    const p = join(web, f);
    h.update(`${f}\n`);
    if (existsSync(p)) h.update(readFileSync(p));
  }
  return h.digest('hex');
};

const installed = existsSync(vite) && existsSync(stamp) && readFileSync(stamp, 'utf8').trim() === lockHash();
if (installed) console.log('node_modules is up to date');
else {
  run('npm', ['install'], process.platform === 'win32');
  // After the install: it may have rewritten the lockfile.
  writeFileSync(stamp, `${lockHash()}\n`, { encoding: 'utf-8' });
}

run(process.execPath, [join(web, 'scripts', 'build-wasm.mjs')]);

// Vite in this process's place: its output and exit are the server's.
const server = spawnSync(process.execPath, [vite, ...process.argv.slice(2)], { cwd: web, stdio: 'inherit' });
process.exit(server.status ?? 1);
