// `node gen.mjs [<id> …] [--pool=N]`: draws 025's painterly candidates with Codex CLI's
// built-in image tool on Matt's ChatGPT login (020's route, ../020_building_card_art/gen.mjs).
// Ids: `icon-<nav tile>` (a small painterly nav icon), `horizon-<direction>` (a wide lunar
// horizon strip), `progress-<step>-<horizon>` (an edit of an earlier draw, attached as an
// image: `pad` edits the horizon, `liftoff` edits `progress-pad-<horizon>`). With no ids,
// every icon and horizon not yet drawn. Originals go to the shared, uncommitted
// art/nav_horizon/raw/<id>.png, an earlier draw kept as <id>.try<n>.png, one log line each
// in log.jsonl. Stops at the first usage-limit message; exits 1 if any draw failed.
import { spawn } from 'node:child_process';
import { appendFile, copyFile, mkdir, readdir, readFile, rename, stat } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const RAW = path.join(REPO, 'art', 'nav_horizon', 'raw');
const CODEX_HOME = process.env.CODEX_HOME ?? path.join(os.homedir(), '.codex');
const TIMEOUT_MS = 6 * 60_000;
const GRACE_MS = 5_000;

const argv = process.argv.slice(2);
const pool = Number(argv.find((a) => a.startsWith('--pool='))?.slice(7) ?? 3);
const p = JSON.parse(await readFile(path.join(HERE, 'prompts.json'), 'utf8'));
const all = [...Object.keys(p.icons).map((t) => `icon-${t}`), ...Object.keys(p.horizons).map((h) => `horizon-${h}`)];
const ids = argv.filter((a) => !a.startsWith('--'));
const todo = ids.length ? ids : all.filter((i) => !existsSync(path.join(RAW, `${i}.png`)));
await mkdir(RAW, { recursive: true });

/** The prompt for one id and the image it edits, if any. */
function job(id) {
  const head = ['Use your built-in image_gen tool ($imagegen) exactly once to generate the image below.',
    'Do not read any files, run any commands, or try to save the image anywhere: just generate it, then stop.'];
  let m;
  if ((m = id.match(/^icon-(\w+)$/)) && p.icons[m[1]]) {
    const body = [p.preamble, p.painterly, p.sun, p.icon_framing, `The object: ${p.icons[m[1]]}`, p.icon_closing];
    return { text: [...head, 'Image shape: square, 1:1, with a transparent background.', '', body.join(' ')].join('\n') };
  }
  if ((m = id.match(/^horizon-(\w+)$/)) && p.horizons[m[1]]) {
    const body = [p.preamble, p.painterly, p.sun, p.horizon_framing, `The scene: ${p.horizons[m[1]]}`];
    const alpha = p.horizons[m[1]].includes('transparent') ? ', with a transparent background' : '';
    return { text: [...head, `Image shape: wide landscape, 3:2${alpha}.`, '', body.join(' ')].join('\n') };
  }
  if ((m = id.match(/^progress-(pad|liftoff)-(\w+)$/)) && p.horizons[m[2]]) {
    const base = m[1] === 'pad' ? `horizon-${m[2]}` : `progress-pad-${m[2]}`;
    const image = path.join(RAW, `${base}.png`);
    if (!existsSync(image)) throw new Error(`gen: ${id} needs ${image}`);
    const alpha = p.horizons[m[2]].includes('transparent') ? ', with a transparent background' : '';
    const body = [`The attached image is a painted lunar horizon strip for the game.`, p.progress[m[1]], p.painterly];
    return { text: [...head, `Image shape: wide landscape, 3:2${alpha}.`, '', body.join(' ')].join('\n'), image };
  }
  throw new Error(`gen: unknown id ${id}`);
}
// Progress stages each edit the draw before them: they run last, one at a time, in order
// (pad before liftoff), after everything else; their source is checked when they start.
const isStage = (i) => i.startsWith('progress-');
for (const i of todo) if (!isStage(i)) job(i);
const stages = todo.filter(isStage).sort((a, b) => (a.includes('-pad-') ? 0 : 1) - (b.includes('-pad-') ? 0 : 1));

function killTree(child) {
  if (process.platform !== 'win32') return child.kill();
  spawn('taskkill', ['/pid', String(child.pid), '/t', '/f'], { stdio: 'ignore' }).on('error', () => child.kill());
}

async function runCodex({ text, image }) {
  const workdir = path.join(REPO, 'out', '025_nav_horizon_art_r1', 'codex');
  await mkdir(workdir, { recursive: true });
  const started = Date.now();
  const attach = image ? ` -i "${image}"` : '';
  const child = spawn(`codex exec --json --skip-git-repo-check -s read-only -C "${workdir}"${attach} -`, { shell: true, stdio: ['pipe', 'pipe', 'pipe'] });
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
  const run = await runCodex(job(id));
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
const queue = todo.filter((i) => !isStage(i));
await Promise.all(
  Array.from({ length: pool }, async () => {
    while (queue.length && !stop) await one(queue.shift());
  }),
);
queue.push(...stages);
while (queue.length && !stop && !failed) await one(queue.shift());
console.log(`gen: done, ${todo.length - failed - queue.length} of ${todo.length} drawn, ${failed} failed, ${queue.length} not started`);
if (failed || queue.length) process.exitCode = 1;
