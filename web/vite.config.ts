import { spawn } from 'node:child_process';
import { dirname, resolve } from 'node:path';
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
        if (f.startsWith(resolve(repo, 'configs'))) server.ws.send({ type: 'full-reload' });
        else if (/[\\/](crates|content|scenarios)[\\/]/.test(f) && !/[\\/](pkg|target)[\\/]/.test(f) && /\.(rs|json|toml)$/.test(f))
          rebuild(server);
      };
      server.watcher.on('change', onChange);
      server.watcher.on('add', onChange);
    },
  };
}

export default defineConfig({
  base: './',
  plugins: [engineWatch()],
  server: {
    // configs/ lives beside web/.
    fs: { allow: [repo] },
  },
});
