// `node gen.mjs [pad|liftoff|climb|hop …]`: redraws 025's horizon progress stages on the
// picked `earthlight` strip with Codex CLI's image tool (025's route, ../025_nav_horizon_art_r1/gen.mjs).
// `pad` edits 025's art/nav_horizon/raw/horizon-earthlight.png (read only); `liftoff`,
// `climb` and `hop` (three takes of the airborne rocket) each edit this round's pad draw. Each stage is an edit of the one before (attached with
// `codex exec -i`), so the base stays put. Originals go to the shared art/small_icons/raw/
// as progress-<stage>-earthlight.png (an earlier draw kept as .try<n>.png), one log line
// each in log.jsonl. Stages run in order, one at a time; stops at the first failure.
import { spawn } from 'node:child_process';
import { appendFile, copyFile, mkdir, readdir, readFile, rename, stat } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const RAW = path.join(REPO, 'art', 'small_icons', 'raw');
const BASE = path.join(REPO, 'art', 'nav_horizon', 'raw', 'horizon-earthlight.png');
const CODEX_HOME = process.env.CODEX_HOME ?? path.join(os.homedir(), '.codex');
const TIMEOUT_MS = 6 * 60_000;

const p = JSON.parse(await readFile(path.join(HERE, 'prompts.json'), 'utf8'));
const args = process.argv.slice(2);
const stages = (args.length ? args : ['pad', 'liftoff', 'climb', 'hop']).sort((a, b) => (a === 'pad' ? 0 : 1) - (b === 'pad' ? 0 : 1));
await mkdir(RAW, { recursive: true });

function job(stage) {
  if (!p.progress[stage]) throw new Error(`gen: unknown stage ${stage}`);
  const image = stage === 'pad' ? BASE : path.join(RAW, 'progress-pad-earthlight.png');
  if (!existsSync(image)) throw new Error(`gen: ${stage} needs ${image}`);
  const text = [
    'Use your built-in image_gen tool ($imagegen) exactly once to generate the image below.',
    'Do not read any files, run any commands, or try to save the image anywhere: just generate it, then stop.',
    'Image shape: wide landscape, 3:2, with a transparent background.',
    '',
    ['The attached image is a painted lunar horizon strip for the game, at night.', p.progress[stage], p.night, p.painterly].join(' '),
  ].join('\n');
  return { text, image };
}

async function runCodex({ text, image }) {
  const workdir = path.join(REPO, 'out', '026_icon_art_r1', 'codex');
  await mkdir(workdir, { recursive: true });
  const started = Date.now();
  const child = spawn(`codex exec --json --skip-git-repo-check -s read-only -C "${workdir}" -i "${image}" -`, { shell: true, stdio: ['pipe', 'pipe', 'pipe'] });
  child.stdin.end(text);
  let out = '';
  let err = '';
  child.stdout.on('data', (d) => (out += d.toString('utf8')));
  child.stderr.on('data', (d) => (err += d.toString('utf8')));
  const timer = setTimeout(() => spawn('taskkill', ['/pid', String(child.pid), '/t', '/f'], { stdio: 'ignore' }), TIMEOUT_MS);
  const code = await new Promise((resolve) => child.on('close', resolve));
  clearTimeout(timer);
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
  if (run.thread) {
    const dir = path.join(CODEX_HOME, 'generated_images', run.thread);
    if (existsSync(dir)) {
      const pngs = (await readdir(dir)).filter((f) => f.endsWith('.png'));
      const timed = await Promise.all(pngs.map(async (f) => ({ f, t: (await stat(path.join(dir, f))).mtimeMs })));
      const newest = timed.sort((a, b) => b.t - a.t)[0];
      if (newest) run.image = path.join(dir, newest.f);
    }
  }
  if (!run.image && !run.message) run.message = `codex exited ${code}: ${err.trim().split('\n').slice(-3).join(' | ')}`;
  return run;
}

for (const stage of stages) {
  const id = `progress-${stage}-earthlight`;
  const run = await runCodex(job(stage));
  await appendFile(path.join(RAW, 'log.jsonl'), `${JSON.stringify({ id, at: new Date().toISOString(), seconds: run.seconds, thread: run.thread, message: run.image ? undefined : run.message })}\n`);
  if (!run.image) {
    console.error(`gen: ${id}: no image after ${run.seconds}s. ${run.message ?? ''}`);
    process.exitCode = 1;
    break;
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
console.log('gen: done');
