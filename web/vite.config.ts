import { execFileSync, spawn } from 'node:child_process';
import { appendFileSync, existsSync, mkdirSync } from 'node:fs';
import type { IncomingMessage, ServerResponse } from 'node:http';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig, type Plugin, type ViteDevServer } from 'vite';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '..');

/**
 * Dev only: rebuilds the wasm when engine sources, content or scenarios change, then
 * reloads the page; a config edit just reloads. A failed build keeps the last good
 * engine (the error is in this terminal).
 */
function engineWatch(): Plugin {
  let running = false;
  let again = false;
  const rebuild = (server: ViteDevServer) => {
    if (running) {
      again = true;
      return;
    }
    running = true;
    server.config.logger.info('engine changed: rebuilding wasm…', { timestamp: true });
    const p = spawn(process.execPath, [resolve(here, 'scripts/build-wasm.mjs')], { stdio: 'inherit' });
    p.on('exit', (code) => {
      running = false;
      if (again) {
        again = false;
        rebuild(server);
      } else if (code === 0) {
        server.ws.send({ type: 'full-reload' });
      } else {
        server.config.logger.error('wasm build failed; the page keeps the last good engine');
      }
    });
  };
  return {
    name: 'hh-engine-watch',
    apply: 'serve',
    configureServer(server) {
      const dirs = ['crates', 'content', 'scenarios', 'configs'].map((d) => resolve(repo, d));
      server.watcher.add(dirs);
      const onChange = (file: string) => {
        const f = resolve(file);
        // The page reads configs/baseline.json and the wasm of hh-core + hh-wasm (with
        // content/ and scenarios/ embedded): an hh-sim or configs/sim edit changes neither.
        if (f === resolve(repo, 'configs', 'baseline.json')) server.ws.send({ type: 'full-reload' });
        else if (/[\\/](crates[\\/]hh-(core|wasm)|content|scenarios)[\\/]/.test(f) && !/[\\/](pkg|target)[\\/]/.test(f) && /\.(rs|json|toml)$/.test(f))
          rebuild(server);
      };
      server.watcher.on('change', onChange);
      server.watcher.on('add', onChange);
    },
  };
}

/** Where play logs go: `HH_PLAYLOG_DIR`, else the synced folder on Matt's machine,
 * else `out/playlogs/` in the repo. A browser under automation (`bot_` careers: the
 * e2e) logs to `out/playlogs/` unless `HH_PLAYLOG_DIR` says otherwise, so the synced
 * folder holds only people's play. */
const SYNCED = 'C:\\Code\\gdrive-sync\\helium-hustle';
const localDir = resolve(repo, 'out', 'playlogs');
const playlogDir = process.env.HH_PLAYLOG_DIR
  ? resolve(process.env.HH_PLAYLOG_DIR)
  : existsSync(SYNCED)
    ? join(SYNCED, 'playlogs')
    : localDir;
const dirFor = (career: string) => (career.startsWith('bot_') && !process.env.HH_PLAYLOG_DIR ? localDir : playlogDir);

/** The repo commit, `+dirty` with uncommitted changes. */
function commit(): string {
  try {
    const head = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repo, encoding: 'utf8' }).trim();
    const dirty = execFileSync('git', ['status', '--porcelain'], { cwd: repo, encoding: 'utf8' }).trim() !== '';
    return dirty ? `${head}+dirty` : head;
  } catch {
    return 'unknown';
  }
}

/**
 * The play log endpoint (dev server and preview): `POST /__playlog` with
 * `{career, lines}` appends the lines to `<dir>/<career>.jsonl`, stamping a header line
 * with the repo commit; `GET /__playlog` answers `{dir}`. One request at a time per page
 * keeps a file in order (src/playlog.ts).
 */
function playlogServer(): Plugin {
  const handle = (req: IncomingMessage, res: ServerResponse) => {
    if (req.method === 'GET') {
      // `?career=<id>`: that career's folder (the e2e reads its own log back).
      const career = new URL(req.url ?? '', 'http://x').searchParams.get('career') ?? '';
      res.setHeader('Content-Type', 'application/json');
      res.end(JSON.stringify({ dir: career ? dirFor(career) : playlogDir }));
      return;
    }
    if (req.method !== 'POST') {
      res.statusCode = 405;
      res.end();
      return;
    }
    let body = '';
    req.setEncoding('utf8');
    req.on('data', (c: string) => (body += c));
    req.on('end', () => {
      let career: string;
      let out: string[];
      try {
        const batch = JSON.parse(body) as { career: string; lines: string[] };
        career = batch.career;
        if (!/^[\w.-]+$/.test(career) || !Array.isArray(batch.lines)) throw new Error('bad play log batch');
        out = batch.lines.map((l) => {
          const v = JSON.parse(l) as Record<string, unknown>;
          return v.kind === 'header' ? JSON.stringify({ ...v, commit: commit() }) : JSON.stringify(v);
        });
      } catch (e) {
        // A batch that can never be taken: the page drops it.
        res.statusCode = 400;
        res.end(String(e));
        return;
      }
      try {
        const dir = dirFor(career);
        mkdirSync(dir, { recursive: true });
        appendFileSync(join(dir, `${career}.jsonl`), out.map((l) => `${l}\n`).join(''), 'utf8');
        res.statusCode = 204;
        res.end();
      } catch (e) {
        // The folder can't be written now (an unmounted drive): the page keeps the
        // batch and retries.
        res.statusCode = 500;
        res.end(String(e));
      }
    });
  };
  return {
    name: 'hh-playlog',
    configureServer(server) {
      server.middlewares.use('/__playlog', handle);
      server.config.logger.info(`play logs: ${playlogDir}`);
    },
    configurePreviewServer(server) {
      server.middlewares.use('/__playlog', handle);
    },
  };
}

export default defineConfig({
  base: './',
  plugins: [engineWatch(), playlogServer()],
  // TSX renders through Preact's automatic JSX runtime.
  oxc: { jsx: { runtime: 'automatic', importSource: 'preact' } },
  server: {
    // configs/ lives beside web/.
    fs: { allow: [repo] },
  },
});
