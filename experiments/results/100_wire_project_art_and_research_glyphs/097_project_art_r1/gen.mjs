// `node gen.mjs [<project>-<option> …] [--pool=N]`: draws project card candidates in
// 020's cut34 building-card style with Codex CLI's image tool (020's gen.mjs, adapted).
// With no ids, every option in prompts.json not yet drawn. Originals go to
// art/projects_r1/raw/<project>-<option>.png (the shared art/ folder), an earlier draw kept
// as .try<n>.png, one log line each in log.jsonl. Stops at the first usage-limit message.
import { spawn } from 'node:child_process';
import { appendFile, copyFile, mkdir, readdir, readFile, rename, stat } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..', '..');
const RAW = path.join(REPO, 'art', 'projects_r1', 'raw');
const CODEX_HOME = process.env.CODEX_HOME ?? path.join(os.homedir(), '.codex');
const TIMEOUT_MS = 6 * 60_000;
const GRACE_MS = 5_000;

const argv = process.argv.slice(2);
const pool = Number(argv.find((a) => a.startsWith('--pool='))?.slice(7) ?? 3);
const p = JSON.parse(await readFile(path.join(HERE, 'prompts.json'), 'utf8'));
const all = {};
for (const [pid, pr] of Object.entries(p.projects)) for (const [k, o] of Object.entries(pr.options)) all[`${pid}-${k}`] = o;
const ids = argv.filter((a) => !a.startsWith('--'));
const todo = ids.length ? ids : Object.keys(all).filter((id) => !existsSync(path.join(RAW, `${id}.png`)));
for (const id of todo) if (!all[id]) throw new Error(`gen: unknown option ${id}`);
await mkdir(RAW, { recursive: true });

function prompt(id) {
  const o = all[id];
  const body = o.float
    ? [p.preamble, p.painterly, p.touch_float, p.sun, p.cut_framing_float, `The subject: ${o.subject}`, p.closing]
    : [p.preamble, p.painterly, p.touch, p.sun, p.cut_framing, `The subject: ${o.subject}`, p.closing];
  return [
    'Use your built-in image_gen tool ($imagegen) exactly once to generate the image below.',
    'Do not read any files, run any commands, or try to save the image anywhere: just generate it, then stop.',
    'Image shape: square, 1:1, with a transparent background.',
    '',
    body.join(' '),
  ].join('\n');
}

function killTree(child) {
  if (process.platform !== 'win32') return child.kill();
  spawn('taskkill', ['/pid', String(child.pid), '/t', '/f'], { stdio: 'ignore' }).on('error', () => child.kill());
}

async function runCodex(text) {
  const workdir = path.join(REPO, 'out', '097_project_art_r1', 'codex');
  await mkdir(workdir, { recursive: true });
  const started = Date.now();
  const child = spawn(`codex exec --json --skip-git-repo-check -s read-only -C "${workdir}" -`, { shell: true, stdio: ['pipe', 'pipe', 'pipe'] });
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
  const run = await runCodex(prompt(id));
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
