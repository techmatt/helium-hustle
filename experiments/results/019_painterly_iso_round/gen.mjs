// `node gen.mjs [<variant>-<building> …] [--pool=N]`: draws round 2's building art with
// Codex CLI's built-in image tool on Matt's ChatGPT login (no API key): 018's route
// (../018_building_art_round/gen.mjs), plus the fixed patch guide on `-i` for the
// isometric variants (AIComplete's frame route). With no ids, every variant × building in
// prompts.json. Originals go to art/buildings/r2/raw/<id>.png (the shared, uncommitted
// art/ folder), an earlier draw kept as <id>.try<n>.png, one log line each in log.jsonl.
// Run guide.py first. Stops at the first usage-limit message; exits 1 if any draw failed.
import { spawn } from 'node:child_process';
import { appendFile, copyFile, mkdir, readdir, readFile, rename, stat } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const R2 = path.join(REPO, 'art', 'buildings', 'r2');
const RAW = path.join(R2, 'raw');
const CODEX_HOME = process.env.CODEX_HOME ?? path.join(os.homedir(), '.codex');
const TIMEOUT_MS = 6 * 60_000;
const GRACE_MS = 5_000;

const argv = process.argv.slice(2);
const pool = Number(argv.find((a) => a.startsWith('--pool='))?.slice(7) ?? 3);
const p = JSON.parse(await readFile(path.join(HERE, 'prompts.json'), 'utf8'));
const all = Object.keys(p.variants).flatMap((v) => Object.keys(p.buildings).map((b) => `${v}-${b}`));
const ids = argv.filter((a) => !a.startsWith('--'));
const todo = ids.length ? ids : all;
for (const id of todo) {
  const [v, b] = id.split('-');
  if (!p.variants[v] || !p.buildings[b]) throw new Error(`gen: unknown id ${id}`);
  const g = p.variants[v].guide;
  if (g && !existsSync(path.join(R2, 'guides', `${g}.png`))) throw new Error(`gen: guide ${g} missing: run guide.py`);
}
await mkdir(RAW, { recursive: true });

/** The full prompt for one variant and building. */
export function prompt(v, b) {
  const st = p.variants[v];
  const body = [p.preamble, p.painterly, st.touch, p.sun, st.shadow, st.guide ? p.iso_camera : p.cut_framing, `The subject: ${p.buildings[b]}`, p.closing];
  return [
    'Use your built-in image_gen tool ($imagegen) exactly once to generate the image below.',
    'Do not read any files, run any commands, or try to save the image anywhere: just generate it, then stop.',
    st.guide ? "Image shape: square, 1:1, the attached guide's shape, with a transparent background." : 'Image shape: square, 1:1, with a transparent background.',
    '',
    body.filter(Boolean).join(' '),
  ].join('\n');
}

function killTree(child) {
  if (process.platform !== 'win32') return child.kill();
  spawn('taskkill', ['/pid', String(child.pid), '/t', '/f'], { stdio: 'ignore' }).on('error', () => child.kill());
}

async function runCodex(text, guide) {
  const workdir = path.join(REPO, 'out', '019_painterly_iso_round', 'codex');
  await mkdir(workdir, { recursive: true });
  const started = Date.now();
  const child = spawn(`codex exec ${guide ? `-i "${guide}" ` : ''}--json --skip-git-repo-check -s read-only -C "${workdir}" -`, { shell: true, stdio: ['pipe', 'pipe', 'pipe'] });
  child.stdin.end(text);
  let out = '';
  let err = '';
  child.stdout.on('data', (d) => (out += d.toString('utf8')));
  child.stderr.on('data', (d) => (err += d.toString('utf8')));
  let timedOut = false;
  const timer = setTimeout(() => {
    timedOut = true;
    killTree(child);
  }, TIMEOUT_MS);
  let grace;
  const code = await new Promise((resolve) => {
    child.on('close', resolve);
    child.on('exit', (c) => (grace = setTimeout(() => resolve(c), GRACE_MS)));
  });
  clearTimeout(timer);
  clearTimeout(grace);
  const run = { seconds: Math.round((Date.now() - started) / 100) / 10 };
  for (const line of out.split('\n')) {
    let ev;
    try {
      ev = JSON.parse(line);
    } catch {
      continue;
    }
    if (ev.type === 'thread.started') run.thread = ev.thread_id;
    if (ev.item?.type === 'agent_message' && ev.item.text) run.message = ev.item.text.trim();
    const msg = ev.message ?? ev.error?.message;
    if ((ev.type === 'error' || ev.type === 'turn.failed') && msg) run.message = msg;
  }
  const limit = `${run.message ?? ''}\n${err}`.match(/[^\n]*(usage limit|rate limit|limit reached|try again (at|in))[^\n]*/i);
  if (limit) run.limit = limit[0].trim();
  if (run.thread) {
    const dir = path.join(CODEX_HOME, 'generated_images', run.thread);
    if (existsSync(dir)) {
      const pngs = (await readdir(dir)).filter((f) => f.endsWith('.png'));
      const timed = await Promise.all(pngs.map(async (f) => ({ f, t: (await stat(path.join(dir, f))).mtimeMs })));
      const newest = timed.sort((a, b) => b.t - a.t)[0];
      if (newest) run.image = path.join(dir, newest.f);
    }
  }
  if (!run.image && timedOut) run.message = `codex stopped after ${TIMEOUT_MS / 1000}s`;
  if (!run.image && code !== 0 && !run.message) run.message = `codex exited ${code}: ${err.trim().split('\n').slice(-3).join(' | ')}`;
  return run;
}

let stop = false;
let failed = 0;
async function one(id) {
  const [v, b] = id.split('-');
  const g = p.variants[v].guide;
  const run = await runCodex(prompt(v, b), g ? path.join(R2, 'guides', `${g}.png`) : null);
  await appendFile(path.join(RAW, 'log.jsonl'), `${JSON.stringify({ id, at: new Date().toISOString(), seconds: run.seconds, thread: run.thread, limit: run.limit, message: run.image ? undefined : run.message })}\n`);
  if (run.limit) {
    stop = true;
    console.error(`gen: usage limit: ${run.limit}`);
  }
  if (!run.image) {
    failed++;
    console.error(`gen: ${id}: no image after ${run.seconds}s. ${run.message ?? ''}`);
    return;
  }
  const dest = path.join(RAW, `${id}.png`);
  if (existsSync(dest)) {
    let n = 1;
    while (existsSync(path.join(RAW, `${id}.try${n}.png`))) n++;
    await rename(dest, path.join(RAW, `${id}.try${n}.png`));
  }
  await copyFile(run.image, dest);
  console.log(`gen: ${id}: ${run.seconds}s`);
}
const queue = [...todo];
await Promise.all(
  Array.from({ length: pool }, async () => {
    while (queue.length && !stop) await one(queue.shift());
  }),
);
console.log(`gen: done, ${todo.length - failed - queue.length} of ${todo.length} drawn, ${failed} failed, ${queue.length} not started`);
if (failed || queue.length) process.exitCode = 1;
