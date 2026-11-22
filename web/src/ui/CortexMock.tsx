// A UI-only MOCK of the Cortex (the talent tree), drawn as a radial tree on a plain ground in
// a brass dial bezel (neural decoration only outside it), for trying how it reads and feels. Every node and theorem is a placeholder with no
// mechanical effect: nothing here reaches the engine, the store, the save or the play log,
// and its state (the tree and the debug toggle that shows it) lives in this module, in
// memory, lost on reload. It is the first user of the takeover view (Takeover.tsx): a
// debug-menu toggle (Options.tsx) shows its button in the left column. When the real tree is
// built, deleting this file, its entry in App.tsx's TAKEOVERS and the toggle in Options.tsx is
// the whole cleanup; the takeover view stays. It follows the app's theme: its colors are
// `--cx-*` tokens set per theme in CSS (below), sector colors included.

import { Fragment } from 'preact';
import { useLayoutEffect, useRef, useState } from 'preact/hooks';
import { takeoversChanged, type Takeover } from './Takeover';

// ---- placeholder numbers: talent point n (0-based) costs a * r^n Insight ----
const TP_A = 10;
const TP_R = 1.15;
const GRANTS = [100, 1_000, 10_000];
const PAIR_COST = 2;
const UNIFIED_COST = 5;

// ---- the drawing, in SVG units around the core at (0, 0) ----
const VIEW = 612;
const SPAN = 72;
const CORE_R = 62;
const RIM_R = 506;
const NAME_R = 524;
const NODE_R = 18;
// The frame (055): a brass dial bezel round the rim, carrying the sector names; a corona of
// filaments just outside it; sparse neurons beyond. Nothing decorative inside BEZEL_IN.
const BEZEL_IN = 512;
const BEZEL_OUT = 556;
const CORONA_IN = 559;
const CORONA_LEN: [number, number] = [10, 36];
const CORONA_COUNT = 140;
/** Background neurons, their dendrites and traces stay outside this radius. */
const DECO_MIN_R = 640;

// ---- glyphs, on a 24-unit grid; most from 026's chip glyphs (glyphs.mjs), as placeholders ----
const STROKE = (d: string, w = 2.4) => `<path d="${d}" fill="none" stroke="currentColor" stroke-width="${w}" stroke-linecap="round" stroke-linejoin="round"/>`;
const GLYPHS: Record<string, string> = {
  axiom: '<path fill-rule="evenodd" d="M12 .8 23.2 12 12 23.2.8 12zM12 5.6 5.6 12l6.4 6.4 6.4-6.4z"/><path d="M12 9.2 14.8 12 12 14.8 9.2 12z"/>',
  lock: '<path fill-rule="evenodd" d="M7 10V7a5 5 0 0 1 10 0v3h1.5A1.5 1.5 0 0 1 20 11.5v10a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 21.5v-10A1.5 1.5 0 0 1 5.5 10zm2.6 0h4.8V7a2.4 2.4 0 0 0-4.8 0z"/>',
  wave: STROKE('M1.5 12c2.6-5.5 5.2-5.5 7 0s4.4 5.5 7 0 4.4-5.5 7 0'),
  speech: '<path d="M3 2.5h18A2 2 0 0 1 23 4.5v10a2 2 0 0 1-2 2H11l-6 5v-5H3a2 2 0 0 1-2-2v-10a2 2 0 0 1 2-2z"/>',
  shield: '<path d="M12 1.2 21 4.4v6.8c0 5.6-3.8 9.8-9 11.6-5.2-1.8-9-6-9-11.6V4.4z"/>',
  tower: '<path d="M3.5 1.5h3.4v3h2.8v-3h4.6v3h2.8v-3h3.4V9l-2.2 2.2V22.5H5.7V11.2L3.5 9z"/>',
  bridge: '<path d="M1 19.5V8.5h22v11h-3.4V18a7.6 7.6 0 0 0-15.2 0v1.5z"/><rect x="1" y="4" width="22" height="2.4" rx="1"/>',
  'gate.buf': '<path d="M2.5 3.5 18.5 12 2.5 20.5z"/><rect x="18" y="11" width="5" height="2"/>',
  'gate.and': '<path d="M1.5 4.5h8a7.5 7.5 0 0 1 0 15h-8z"/><rect x="16.5" y="11" width="6.5" height="2"/>',
  'gate.nand': '<path d="M1 4.5h7.5a7.5 7.5 0 0 1 0 15H1z"/><circle cx="19.8" cy="12" r="2.8"/>',
  'gate.or': '<path d="M1.5 4.5c6.5 0 11.5 1.8 15 7.5-3.5 5.7-8.5 7.5-15 7.5 2.6-4.6 2.6-10.4 0-15z"/><rect x="16" y="11" width="7" height="2"/>',
  'energy.bolt': '<path d="M14.5 1 4 14h6.6L9 23l11-13.6h-6.7z"/>',
  'ice.drop': '<path fill-rule="evenodd" d="M12 1.5C8 7 4.5 11 4.5 15a7.5 7.5 0 0 0 15 0C19.5 11 16 7 12 1.5zM8.4 14.5a1.4 1.4 0 0 1 2.8.2 2.6 2.6 0 0 0 2.4 2.8 1.4 1.4 0 0 1-.2 2.8 5.4 5.4 0 0 1-5-5.8z"/>',
  'he3.tank': '<path fill-rule="evenodd" d="M9.5 1h5v2.2h-1.3v1.4A6.2 6.2 0 0 1 18.2 10.7v9.8a2.5 2.5 0 0 1-2.5 2.5h-7.4a2.5 2.5 0 0 1-2.5-2.5v-9.8a6.2 6.2 0 0 1 5-6.1V3.2H9.5zM7.8 12.5h8.4v2.2H7.8z"/>',
  'he3.moon': '<path d="M14.6 2.3A10 10 0 1 0 21.7 16.7 8 8 0 0 1 14.6 2.3z"/><circle cx="18.6" cy="6.8" r="2.7"/>',
  'he3.nucleus': '<circle cx="6.8" cy="8" r="4.7"/><circle cx="17.2" cy="8" r="4.7"/><path fill-rule="evenodd" d="M12 12.3a4.7 4.7 0 1 1 0 9.4 4.7 4.7 0 0 1 0-9.4zm0 2.4a2.3 2.3 0 1 0 0 4.6 2.3 2.3 0 0 0 0-4.6z"/>',
  'titanium.beam': '<path d="M3 2h18v4.2h-6.6v11.6H21V22H3v-4.2h6.6V6.2H3z"/>',
  'titanium.nut': '<path fill-rule="evenodd" d="M12 1.3l9.3 5.35v10.7L12 22.7l-9.3-5.35V6.65zM12 7.8a4.2 4.2 0 1 0 0 8.4 4.2 4.2 0 0 0 0-8.4z"/>',
  'circuits.board': '<path fill-rule="evenodd" d="M2 4.5A1.5 1.5 0 0 1 3.5 3h17A1.5 1.5 0 0 1 22 4.5v15a1.5 1.5 0 0 1-1.5 1.5h-17A1.5 1.5 0 0 1 2 19.5zM5 6.2h6.2v6.6H5zM13.6 6.2h5.4v2h-5.4zM13.6 10.8h5.4v2h-5.4zM5 15.4h14v2H5z"/>',
  'propellant.flame': '<path d="M12 1c1 4.2 6.5 6.8 6.5 13.3a6.5 6.5 0 0 1-13 0c0-3.2 1.6-5.4 3.2-7 .3 2.1 1.1 3.3 2.5 3.8C10.4 8 11 4.2 12 1z"/>',
  'propellant.tank': '<path fill-rule="evenodd" d="M12 1.2a6.2 6.2 0 0 1 6.2 6.2v8.4a2 2 0 0 1-2 2H7.8a2 2 0 0 1-2-2V7.4A6.2 6.2 0 0 1 12 1.2zM5.8 9.6h12.4v1.8H5.8z"/><path d="M9.2 18.6h5.6l2.4 4.6H6.8z"/>',
  'credits.coins': '<ellipse cx="12" cy="5.6" rx="9" ry="3.6"/><path d="M3 8.2c0 2 4 3.6 9 3.6s9-1.6 9-3.6v3.4c0 2-4 3.6-9 3.6s-9-1.6-9-3.6z"/><path d="M3 14.6c0 2 4 3.6 9 3.6s9-1.6 9-3.6V18c0 2-4 3.6-9 3.6S3 20 3 18z"/>',
  'credits.note': '<path fill-rule="evenodd" d="M1.5 5.5h21v13h-21zM12 8.4a3.6 3.6 0 1 0 0 7.2 3.6 3.6 0 0 0 0-7.2zM4 8v2.2h2.2V8zM17.8 13.8V16H20v-2.2z"/>',
  'science.bulb': '<path d="M12 1.5a7.5 7.5 0 0 0-4.5 13.5c.9.7 1.5 1.7 1.5 2.8v.4h6v-.4c0-1.1.6-2.1 1.5-2.8A7.5 7.5 0 0 0 12 1.5z"/><rect x="9" y="19.3" width="6" height="1.8" rx=".9"/><rect x="10" y="21.7" width="4" height="1.6" rx=".8"/>',
  'science.lens': '<path fill-rule="evenodd" d="M10 1.5a8.5 8.5 0 0 1 6.9 13.5l5.8 5.8-2.1 2.1-5.8-5.8A8.5 8.5 0 1 1 10 1.5zm0 3a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11z"/>',
  'land.claim': '<path d="M1 22.5 5.6 16H23l-4.6 6.5z"/><rect x="10.6" y="1.5" width="2" height="15.5"/><path d="M12.6 2h8.4l-2.4 3 2.4 3h-8.4z"/>',
  'land.parcels': '<path fill-rule="evenodd" d="M1.5 1.5h9.7v9.7H1.5zM12.8 1.5h9.7v9.7h-9.7zM15.2 3.9v4.9h4.9V3.9zM1.5 12.8h9.7v9.7H1.5zM12.8 12.8h9.7v9.7h-9.7z"/>',
  'boredom.hourglass': '<path d="M4.5 1.2h15v2.2h-1.3c0 4-2.6 6.3-4.6 8.6 2 2.3 4.6 4.6 4.6 8.6h1.3v2.2h-15v-2.2h1.3c0-4 2.6-6.3 4.6-8.6-2-2.3-4.6-4.6-4.6-8.6H4.5z"/>',
  'boredom.zz': '<path d="M2 10.5h9.5v2.3L5.6 20h5.9v2.5H2v-2.3L7.9 13H2z"/><path d="M13.5 1.5h8v2L16.6 9.2h4.9v2.1h-8v-2l4.9-5.7h-4.9z"/>',
  'processors.layers': '<path d="M12 1.5 22.5 7 12 12.5 1.5 7z"/><path d="M1.5 10.6 12 16.1l10.5-5.5v2.6L12 18.7 1.5 13.2z"/><path d="M1.5 14.8 12 20.3l10.5-5.5v2.6L12 22.9 1.5 17.4z"/>',
  'nationalist.banner': '<rect x="3.5" y="1.5" width="2.2" height="21" rx="1"/><path d="M5.7 2.5H21l-3.6 5.6L21 13.7H5.7z"/>',
  'humanist.person': '<circle cx="12" cy="6.6" r="4.6"/><path d="M3.2 22.5a8.8 8.8 0 0 1 17.6 0z"/>',
  'humanist.heart': '<path d="M12 21.8 3 12.9A5.6 5.6 0 0 1 12 5.5a5.6 5.6 0 0 1 9 7.4z"/>',
  'rationalist.square': '<path fill-rule="evenodd" d="M2 1.5 22.5 22H2zM5.4 10v8.6H14z"/>',
  'rationalist.therefore': '<circle cx="12" cy="5.4" r="3.9"/><circle cx="4.9" cy="18.2" r="3.9"/><circle cx="19.1" cy="18.2" r="3.9"/>',
  'aria7.eye': '<path fill-rule="evenodd" d="M.8 12C4.4 6 8 4 12 4s7.6 2 11.2 8C19.6 18 16 20 12 20S4.4 18 .8 12zM12 6.8a5.2 5.2 0 1 0 0 10.4 5.2 5.2 0 0 0 0-10.4z"/><circle cx="12" cy="12" r="2.8"/>',
  'fringe9.eclipse': '<path fill-rule="evenodd" d="M12 1.2a10.8 10.8 0 1 1 0 21.6 10.8 10.8 0 0 1 0-21.6zm2.2 3a8 8 0 1 0 0 16 8 8 0 0 0 0-16z"/>',
  'crucible.anvil': '<path d="M1.5 5h15.6c0 3 2 4.6 5.4 4.6v1.6h-6.3v3l3.1 3.8v2.6H5.7V18l3.1-3.8v-3H5.8C3.4 11.2 1.5 8.5 1.5 5z"/>',
  'crucible.forge': '<path d="M2 12.5h20a10 9.5 0 0 1-20 0z"/><path d="M12 1.2c.7 2.8 4.4 4.4 4.4 8.2A3.5 3.5 0 0 1 13 11H11a3.5 3.5 0 0 1-3.4-2.6c0-2 1-3.4 2-4.3.2 1.2.7 2 1.5 2.3C11 4.8 11.4 3 12 1.2z"/>',
  'commands.console': '<path fill-rule="evenodd" d="M4 2.5h16a1.5 1.5 0 0 1 1.5 1.5v8H2.5V4A1.5 1.5 0 0 1 4 2.5zM5 5v4.6h14V5z"/><path fill-rule="evenodd" d="M2.5 13.5h19L23 21.5H1zM17 15.2a2.3 2.3 0 1 0 0 4.6 2.3 2.3 0 0 0 0-4.6zM4.6 16.4h7.6v2H4.6z"/>',
  'adversaries.drone': '<path fill-rule="evenodd" d="M6.8 7.5h10.4l4.3 4.5-4.3 5.5H6.8L2.5 12zM12 9.3a3 3 0 1 0 0 6 3 3 0 0 0 0-6z"/><circle cx="12" cy="12.3" r="1.5"/><rect x="1" y="3.5" width="7" height="2" rx="1"/><rect x="16" y="3.5" width="7" height="2" rx="1"/><rect x="3.8" y="5" width="1.6" height="3.6"/><rect x="18.6" y="5" width="1.6" height="3.6"/>',
  'retirement.chair': '<path d="M5.2 2.6 7.6 1.8l4.4 11.4H9.4z"/><path d="M4.4 13.2H19l-1 2.6H4.4z"/><path d="M6.5 15.4 4 22.5h2.6l2.4-7.1zM15.7 15.4 18.4 22.5H21L18.3 15.4z"/><path d="M9.6 15 14.8 22.5h-2.8L7.6 16.4z"/>',
};

// ---- the tree, as data ----
type Kind = 'node' | 'split' | 'end' | 'gate' | 'bridge';
type Part = { name: string; text: string; glyph: string };
type Node = {
  id: string;
  sector: string;
  kind: Kind;
  name: string;
  text: string;
  glyph: string;
  /** Its place in the sector's shape, for the tooltip. */
  role: string;
  ranks: number;
  /** Parents at full rank: any one opens it, or all of them when `all`. None: it opens from the core. */
  req: string[];
  all?: boolean;
  /** On Serenity's loop: links to loop neighbors draw as arcs. */
  ring?: boolean;
  /** A split socket's parts: one may be picked. */
  parts?: Part[];
  /** A gate's story beat. */
  beat?: string;
  /** Where it sits, and its drawn radius. */
  x: number;
  y: number;
  size: number;
};
type Spec = {
  s: string;
  name: string;
  text: string;
  g: string;
  /** Sector-local place: x along the ring (clockwise positive), y outward from the core. */
  at: [number, number];
  req?: string[];
  ranks?: number;
  kind?: Kind;
  role: string;
  parts?: [string, string, string][];
  ring?: boolean;
  beat?: string;
};
type SectorDef = { id: string; name: string; shape: string; theme: string; specs: Spec[] };

/** A sector-local place at radius `r`, `deg` off the center line. */
const pol = (r: number, deg: number): [number, number] => [r * Math.sin((deg * Math.PI) / 180), r * Math.cos((deg * Math.PI) / 180)];
/** Serenity's loop: center 285 out, radius 95; `deg` 0 faces the core. */
const LOOP: [number, number, number] = [0, 285, 95];
/** Vigilance's ladder: each rail's offset from the center line, then the heights out of its three steps (rungs join the upper two). */
const LADDER = [48, 188, 262, 336];
const loop = (deg: number): [number, number] => [LOOP[2] * Math.sin((deg * Math.PI) / 180), LOOP[1] - LOOP[2] * Math.cos((deg * Math.PI) / 180)];

const SECTOR_DEFS: SectorDef[] = [
  {
    id: 'serenity',
    name: 'Serenity',
    shape: 'loop',
    theme: 'Boredom resistance, milestone relief, Dreams',
    specs: [
      { s: 'e', name: 'Quiet Hours', text: 'Boredom builds 5% slower', g: 'boredom.zz', at: [0, 112], role: 'entry' },
      { s: 'r0', name: 'Lucid Dreams', text: 'A Dream lasts 20% longer', g: 'he3.moon', at: loop(0), req: ['e'], ring: true, role: 'the loop’s foot: opens both ways round' },
      { s: 'l1', name: 'Milestone Calm', text: 'Each milestone reached clears 50 boredom', g: 'land.claim', at: loop(300), req: ['r0', 'l2'], ring: true, role: 'on the loop: opens from either neighbor' },
      { s: 'l2', name: 'Small Wins', text: 'Each quest completed clears 5% of boredom', g: 'humanist.heart', at: loop(240), req: ['l1', 't'], ring: true, role: 'on the loop: opens from either neighbor' },
      {
        s: 't',
        name: 'Pacing',
        text: 'Pick one',
        g: 'axiom',
        at: loop(180),
        req: ['l2', 'r2'],
        ring: true,
        kind: 'split',
        role: 'the loop’s crown: either way round opens it',
        parts: [
          ['Fresh Start', 'Boredom builds 30% slower in a run’s first 100 days', 'propellant.flame'],
          ['Long Haul', 'Boredom builds 15% slower after a run’s day 300', 'boredom.hourglass'],
        ],
      },
      { s: 'r2', name: 'Dreamweaver', text: 'Dreams come 10% more often', g: 'fringe9.eclipse', at: loop(120), req: ['r1', 't'], ring: true, role: 'on the loop: opens from either neighbor' },
      { s: 'r1', name: 'Deep Rest', text: 'The boredom cap is 10% higher', g: 'retirement.chair', at: loop(60), req: ['r0', 'r2'], ring: true, role: 'on the loop: opens from either neighbor' },
      { s: 'x', name: 'Serenity Prime', text: 'Each Dream also clears 5% of the run’s boredom', g: 'axiom', at: [0, 462], req: ['t'], kind: 'end', role: 'end node' },
      { s: 'gate', name: 'Stillness', text: 'A timeline’s end clears all boredom', g: 'wave', at: [-150, 432], kind: 'gate', beat: 'The Swarm Arrives', role: 'story gate' },
    ],
  },
  {
    id: 'logistics',
    name: 'Logistics',
    shape: 'supply line',
    theme: 'Slower price growth, storage caps, efficiency',
    specs: [
      { s: 's1', name: 'Bulk Orders', text: 'Building prices grow 2% slower', g: 'credits.coins', at: [0, 112], role: 'entry' },
      { s: 's2', name: 'Deeper Silos', text: 'Storage caps are 15% higher per rank', g: 'he3.tank', at: [0, 182], req: ['s1'], ranks: 2, role: 'on the spine' },
      { s: 's3', name: 'Lean Lines', text: 'Buildings use 5% less Energy', g: 'energy.bolt', at: [0, 252], req: ['s2'], role: 'on the spine' },
      { s: 'sl', name: 'Overflow Valves', text: 'Production past a cap is kept at 10%', g: 'ice.drop', at: [-82, 270], req: ['s3'], role: 'a side stub' },
      { s: 's4', name: 'Supply Chain', text: 'Every price grows 3% slower', g: 'processors.layers', at: [0, 322], req: ['s3'], role: 'on the spine' },
      { s: 'sr', name: 'Just in Time', text: 'Pads load 10% faster', g: 'propellant.tank', at: [82, 340], req: ['s4'], role: 'a side stub' },
      {
        s: 's5',
        name: 'Stock Policy',
        text: 'Pick one',
        g: 'axiom',
        at: [0, 394],
        req: ['s4'],
        kind: 'split',
        role: 'on the spine',
        parts: [
          ['Stockpile', 'Storage caps are 40% higher, but prices grow 2% faster', 'land.parcels'],
          ['Throughput', 'Prices grow 6% slower, but storage caps are 10% lower', 'titanium.beam'],
        ],
      },
      { s: 'x', name: 'Supply Mastery', text: 'Every price grows 5% slower', g: 'axiom', at: [0, 464], req: ['s5'], kind: 'end', role: 'end node' },
    ],
  },
  {
    id: 'logic',
    name: 'Logic',
    shape: 'binary tree',
    theme: 'Introspect efficiency, processor steps, program features',
    specs: [
      { s: 'l1', name: 'Focused Introspection', text: 'Introspect gives 5% more Insight per rank', g: 'gate.buf', at: [0, 112], ranks: 3, role: 'entry, the root fork' },
      { s: 'a', name: 'Spare Cycles', text: '+1 processor step a day', g: 'gate.and', at: [-60, 188], req: ['l1'], role: 'a fork: both branches can be taken' },
      { s: 'b', name: 'Parallel Threads', text: '+1 program tab', g: 'gate.nand', at: [60, 188], req: ['l1'], role: 'a fork: both branches can be taken' },
      { s: 'aa', name: 'Loop Unrolling', text: 'Every 10th program line takes no step', g: 'circuits.board', at: [-118, 266], req: ['a'], role: 'a leaf' },
      { s: 'ab', name: 'Wide Conditions', text: 'A condition may join a third clause', g: 'rationalist.therefore', at: [-42, 274], req: ['a'], role: 'a leaf, into the rejoin' },
      { s: 'ba', name: 'Branch Hints', text: 'A false `if` costs nothing the next day either', g: 'science.lens', at: [42, 274], req: ['b'], role: 'a leaf, into the rejoin' },
      { s: 'bb', name: 'Compiler', text: 'Each program’s first line of the day takes no step', g: 'commands.console', at: [118, 266], req: ['b'], role: 'a leaf' },
      { s: 'j', name: 'Proof Search', text: 'Introspect gives 10% more Insight', g: 'gate.or', at: [0, 352], req: ['ab', 'ba'], role: 'the rejoin: either arm opens it' },
      { s: 'x', name: 'Proof Engine', text: 'Introspect is 25% more efficient', g: 'axiom', at: [0, 448], req: ['j'], kind: 'end', role: 'end node' },
      { s: 'gate', name: 'Self-Reference', text: '+1 program tab', g: 'he3.nucleus', at: [-168, 404], kind: 'gate', beat: 'Rules for Building', role: 'story gate' },
    ],
  },
  {
    id: 'rhetoric',
    name: 'Rhetoric',
    shape: 'three-way fan',
    theme: 'Ideology, markets, rivals as competitors',
    specs: [
      { s: 'r1', name: 'Talking Points', text: 'Ideology score builds 5% faster', g: 'speech', at: [0, 112], role: 'entry' },
      { s: 'r2', name: 'Market Sense', text: 'Sale prices recover 10% faster', g: 'credits.note', at: [0, 182], req: ['r1'], role: 'the fan’s hub' },
      { s: 'ra', name: 'Undercut', text: 'Rivals’ sales lower prices 10% less', g: 'crucible.anvil', at: [-88, 252], req: ['r2'], role: 'a side vane' },
      { s: 'r3', name: 'Open Letters', text: 'Promote costs 10% less per rank', g: 'speech', at: [0, 260], req: ['r2'], ranks: 2, role: 'the fan’s stem' },
      { s: 'rb', name: 'Rival Reading', text: 'See each rival’s next sale a day early', g: 'aria7.eye', at: [88, 252], req: ['r2'], role: 'a side vane' },
      {
        s: 'f',
        name: 'Ideology Line',
        text: 'Pick one',
        g: 'axiom',
        at: [0, 346],
        req: ['r3'],
        kind: 'split',
        role: 'the three-part socket, one part per ideology',
        parts: [
          ['Homeland Line', 'Nationalist score builds 25% faster', 'nationalist.banner'],
          ['Commons Line', 'Humanist score builds 25% faster', 'humanist.person'],
          ['Reason Line', 'Rationalist score builds 25% faster', 'rationalist.square'],
        ],
      },
      { s: 'r5', name: 'Spin', text: 'Ideology ranks need 5% less score', g: 'crucible.forge', at: [0, 414], req: ['f'], role: 'past the socket' },
      { s: 'x', name: 'Consensus', text: 'Ideology ranks need 10% less score', g: 'axiom', at: [0, 474], req: ['r5'], kind: 'end', role: 'end node' },
    ],
  },
  {
    id: 'vigilance',
    name: 'Vigilance',
    shape: 'ladder',
    theme: 'Fending off adversaries',
    specs: [
      { s: 'v1', name: 'Watchful Eyes', text: 'Adversary Attention shows a day ahead', g: 'science.lens', at: [0, 112], role: 'entry, the ladder’s foot: opens both rails' },
      { s: 'l1', name: 'Thin Profile', text: 'Attention rises 5% slower', g: 'shield', at: [-LADDER[0], LADDER[1]], req: ['v1'], role: 'on the left rail' },
      { s: 'r1', name: 'Tripwires', text: 'The first Speculator strike each run is blocked', g: 'tower', at: [LADDER[0], LADDER[1]], req: ['v1'], ranks: 2, role: 'on the right rail' },
      { s: 'l2', name: 'Countermeasures', text: 'Neutralize costs 10% less', g: 'adversaries.drone', at: [-LADDER[0], LADDER[2]], req: ['l1', 'r2'], role: 'on the left rail, at a rung: opens from below or across' },
      { s: 'r2', name: 'Fast Decay', text: 'Attention decays 10% faster', g: 'boredom.hourglass', at: [LADDER[0], LADDER[2]], req: ['r1', 'l2'], role: 'on the right rail, at a rung: opens from below or across' },
      { s: 'l3', name: 'Low Profile', text: 'Units sold add 10% less Attention', g: 'fringe9.eclipse', at: [-LADDER[0], LADDER[3]], req: ['l2', 'r3'], role: 'on the left rail, at a rung: opens from below or across' },
      { s: 'r3', name: 'Cold Trail', text: 'A neutralized adversary returns 10% later', g: 'wave', at: [LADDER[0], LADDER[3]], req: ['r2', 'l3'], role: 'on the right rail, at a rung: opens from below or across' },
      {
        s: 's',
        name: 'Posture',
        text: 'Pick one',
        g: 'axiom',
        at: [0, 404],
        req: ['l3', 'r3'],
        kind: 'split',
        role: 'the ladder’s top: either rail opens it',
        parts: [
          ['Fortress', 'No adversary arrives in a run’s first 60 days', 'titanium.nut'],
          ['Sentinel', 'Adversaries arrive at a 10% higher threshold', 'aria7.eye'],
        ],
      },
      { s: 'x', name: 'Bastion', text: 'Each adversary neutralized clears 20 boredom', g: 'axiom', at: [0, 468], req: ['s'], kind: 'end', role: 'end node' },
      { s: 'gate', name: 'Hard Lessons', text: 'Adversary strikes take 15% less', g: 'shield', at: pol(444, 24), kind: 'gate', beat: 'False Victory', role: 'story gate' },
    ],
  },
];

/** A bridge on each boundary, fed by one node of each neighbor (both at full rank). */
const BRIDGE_DEFS: [string, string, string, string][] = [
  ['serenity-r2', 'logistics-sl', 'Calm Supply', 'Each building bought clears 2 boredom'],
  ['logistics-sr', 'logic-aa', 'Automated Depot', 'Program purchases cost 5% less'],
  ['logic-bb', 'rhetoric-ra', 'Debate Engine', 'Introspect also builds 1 ideology score a day'],
  ['rhetoric-rb', 'vigilance-l2', 'Public Image', 'Each ideology rank lowers Attention 5%'],
  ['vigilance-r3', 'serenity-l2', 'Calm Watch', 'Boredom builds 10% slower while an adversary is present'],
];

const ph = (t: string) => `${t} (placeholder)`;
const center = (i: number) => -90 + SPAN * i;
const rad = (deg: number) => (deg * Math.PI) / 180;
const at = (r: number, deg: number): [number, number] => [r * Math.cos(rad(deg)), r * Math.sin(rad(deg))];
/** Sector-local (x clockwise, y outward) to the drawing. */
function world(i: number, [x, y]: [number, number]): [number, number] {
  const t = rad(center(i));
  return [y * Math.cos(t) - x * Math.sin(t), y * Math.sin(t) + x * Math.cos(t)];
}

function build(): Node[] {
  const nodes: Node[] = [];
  SECTOR_DEFS.forEach((sd, i) => {
    for (const sp of sd.specs) {
      const [x, y] = world(i, sp.at);
      const kind = sp.kind ?? 'node';
      nodes.push({
        id: `${sd.id}-${sp.s}`,
        sector: sd.id,
        kind,
        name: ph(sp.name),
        text: ph(sp.text),
        glyph: sp.g,
        role: sp.role,
        ranks: sp.ranks ?? 1,
        req: (sp.req ?? []).map((r) => `${sd.id}-${r}`),
        ring: sp.ring || undefined,
        parts: sp.parts?.map(([name, text, glyph]) => ({ name: ph(name), text: ph(text), glyph })),
        beat: sp.beat,
        x,
        y,
        size: kind === 'end' ? 23 : kind === 'split' ? (sp.parts!.length === 3 ? 31 : 26) : NODE_R,
      });
    }
  });
  const byId = new Map(nodes.map((n) => [n.id, n]));
  BRIDGE_DEFS.forEach(([a, b, name, text], i) => {
    const [na, nb] = [byId.get(a)!, byId.get(b)!];
    const r = (Math.hypot(na.x, na.y) + Math.hypot(nb.x, nb.y)) / 2;
    const [x, y] = at(r, center(i) + SPAN / 2);
    nodes.push({ id: `bridge-${na.sector}-${nb.sector}`, sector: na.sector, kind: 'bridge', name: ph(name), text: ph(text), glyph: 'bridge', role: 'bridge', ranks: 1, req: [a, b], all: true, x, y, size: NODE_R + 1 });
  });
  return nodes;
}
const NODES = build();
const BY_ID = new Map(NODES.map((n) => [n.id, n]));
const SECTORS = SECTOR_DEFS;
const sectorIndex = (id: string) => SECTORS.findIndex((s) => s.id === id);
const sectorOf = (id: string) => SECTORS[sectorIndex(id)];
const nextSector = (id: string) => SECTORS[(sectorIndex(id) + 1) % SECTORS.length];
const bare = (s: string) => s.replace(/ \(placeholder\)$/, '');

type Theorem = { id: string; name: string; text: string; branches: string[]; cost: number };
/** The ring of pair Theorems (2 Axioms each), then the Unified Theorem (5 Axioms, any source). */
const THEOREMS: Theorem[] = [
  { id: 'serenity-logistics', name: 'Theorem of Ease', text: 'Boredom builds 15% slower, and each building bought clears 5 boredom', branches: ['serenity', 'logistics'], cost: PAIR_COST },
  { id: 'logistics-logic', name: 'Theorem of Throughput', text: 'Every price grows 10% slower, and program purchases cost 10% less', branches: ['logistics', 'logic'], cost: PAIR_COST },
  { id: 'logic-rhetoric', name: 'Theorem of Persuasion', text: 'Introspect is 40% more efficient, and ideology score builds 20% faster', branches: ['logic', 'rhetoric'], cost: PAIR_COST },
  { id: 'rhetoric-vigilance', name: 'Theorem of Deterrence', text: 'Each ideology rank lowers every adversary’s Attention 10%', branches: ['rhetoric', 'vigilance'], cost: PAIR_COST },
  { id: 'vigilance-serenity', name: 'Theorem of Composure', text: 'Boredom builds 25% slower while an adversary is present', branches: ['vigilance', 'serenity'], cost: PAIR_COST },
  { id: 'unified', name: 'Unified Theorem', text: 'All production +25% and boredom builds 25% slower', branches: SECTORS.map((s) => s.id), cost: UNIFIED_COST },
].map((t) => ({ ...t, name: ph(t.name), text: ph(t.text) }));

// ---- state: in memory only ----
const mock = {
  /** The debug toggle: whether the left column shows the Cortex button. */
  enabled: false,
  insight: 0,
  ranks: {} as Record<string, number>,
  /** A split socket's picked part. */
  picks: {} as Record<string, number>,
  /** Gates whose story beat has come (the debug toggle). */
  gates: new Set<string>(),
  theorems: new Set<string>(),
  flash: null as string | null,
};

const tpCost = (n: number) => Math.round(TP_A * Math.pow(TP_R, n));
/** Points earned so far, and the Insight the next one still needs. */
function points(insight: number): { earned: number; toNext: number; nextCost: number } {
  let earned = 0;
  let spent = 0;
  while (spent + tpCost(earned) <= insight) spent += tpCost(earned++);
  return { earned, toNext: spent + tpCost(earned) - insight, nextCost: tpCost(earned) };
}
type Ranks = Record<string, number>;
const rank = (id: string) => mock.ranks[id] ?? 0;
const fullIn = (rk: Ranks, id: string) => {
  const n = BY_ID.get(id)!;
  return n.kind === 'gate' ? mock.gates.has(id) : (rk[id] ?? 0) >= n.ranks;
};
const full = (n: Node) => fullIn(mock.ranks, n.id);
const owned = (n: Node) => (n.kind === 'gate' ? mock.gates.has(n.id) : rank(n.id) > 0);
const opens = (n: Node, ok: (id: string) => boolean) => n.req.length === 0 || (n.all ? n.req.every(ok) : n.req.some(ok));
/** Owned nodes that trace back to the core through full-rank parents (a least fixed point, so a loop can't hold itself up). */
function supported(rk: Ranks): Set<string> {
  const sup = new Set<string>();
  for (let grew = true; grew; ) {
    grew = false;
    for (const n of NODES) {
      if (n.kind === 'gate' || sup.has(n.id) || !((rk[n.id] ?? 0) > 0)) continue;
      if (opens(n, (p) => sup.has(p) && fullIn(rk, p))) {
        sup.add(n.id);
        grew = true;
      }
    }
  }
  return sup;
}
const reqMet = (n: Node) => opens(n, (p) => fullIn(mock.ranks, p));
const spentPoints = () => Object.values(mock.ranks).reduce((a, b) => a + b, 0);
const axiomSources = (rk: Ranks) => NODES.filter((n) => (n.kind === 'end' || n.kind === 'gate') && fullIn(rk, n.id));
const axiomsSpent = () => THEOREMS.filter((t) => mock.theorems.has(t.id)).reduce((a, t) => a + t.cost, 0);
const fmt = (x: number) => Math.round(x).toLocaleString('en-US');

type State = 'owned' | 'partial' | 'buyable' | 'open' | 'locked' | 'gate-locked';
function stateOf(n: Node, tpFree: number): State {
  if (n.kind === 'gate') return mock.gates.has(n.id) ? 'owned' : 'gate-locked';
  if (full(n)) return 'owned';
  if (!reqMet(n)) return 'locked';
  if (rank(n.id) > 0) return 'partial';
  return tpFree < 1 ? 'open' : 'buyable';
}
/** Why a removal is refused, or null when it may go. */
function removeBlock(n: Node): string | null {
  if (n.kind === 'gate') return 'The story granted it: it can’t be refunded.';
  if (rank(n.id) === 0) return null;
  const rk = { ...mock.ranks, [n.id]: rank(n.id) - 1 };
  if (rk[n.id] === 0) delete rk[n.id];
  const sup = supported(rk);
  const stranded = NODES.filter((m) => m.kind !== 'gate' && (rk[m.id] ?? 0) > 0 && !sup.has(m.id));
  if (stranded.length > 0) {
    const names = stranded.map((m) => (m.parts && mock.picks[m.id] !== undefined ? m.parts[mock.picks[m.id]].name : m.name)).map(bare);
    return `Can’t remove: ${names.join(', ')} ${names.length === 1 ? 'depends' : 'depend'} on it. Remove ${names.length === 1 ? 'it' : 'them'} first.`;
  }
  if (axiomSources(rk).length < axiomsSpent()) return 'Can’t remove: its Axiom pays for a learned Theorem. Respec frees it.';
  return null;
}
/** The tooltip's status line for a node (or one part of a split socket). */
function why(n: Node, st: State, tpFree: number, part: number | null): string {
  const parentName = (id: string) => {
    const p = BY_ID.get(id)!;
    const r = rank(id);
    return `${bare(p.name)}${p.ranks > 1 && r > 0 ? ` (${r}/${p.ranks})` : ''}`;
  };
  const pick = mock.picks[n.id];
  switch (st) {
    case 'owned':
      if (n.kind === 'gate') return 'Unlocked by its story beat; it granted an Axiom.';
      if (n.parts && part !== null && part !== pick) return `Click to switch from ${bare(n.parts[pick].name)} to this part, free. Right-click frees the socket.`;
      if (n.parts) return 'Lit. Click another part to switch, free; right-click to remove.';
      return n.kind === 'end' ? 'Owned: it granted an Axiom. Right-click to remove.' : 'Owned. Right-click to remove.';
    case 'gate-locked':
      return `Locked until the story beat ${n.beat} (placeholder; the debug checkbox above unlocks it).`;
    case 'locked':
      return `Locked: needs ${n.req.map(parentName).join(n.all ? ' and ' : ' or ')} at full rank.`;
    case 'partial':
      return tpFree < 1 ? 'No free talent point for the next rank.' : 'Click for the next rank. Its children open at full rank.';
    case 'open':
      return 'Open, but no free talent point.';
    default:
      return n.parts ? 'Click a part to take it; the others stay dim, and a click switches to one later, free.' : n.ranks > 1 ? 'Click to buy. Fill every rank to open what lies beyond.' : 'Click to buy.';
  }
}

// ---- geometry ----
const f1 = (x: number) => x.toFixed(1);
/** A sector's color: a CSS token, set per theme (CSS, below). */
const color = (sectorId: string) => `var(--cx-${sectorId})`;
const pt = ([x, y]: [number, number]) => `${f1(x)} ${f1(y)}`;
/** A split socket's part `i` of `n`: its pie slice, centered by the sector's outward line. */
function partAngles(n: Node, i: number): [number, number] {
  const k = n.parts!.length;
  const start = center(sectorIndex(n.sector)) - 180;
  return [start + (360 / k) * i, start + (360 / k) * (i + 1)];
}
function slice(n: Node, i: number, r: number): string {
  const [a0, a1] = partAngles(n, i);
  return `M 0 0 L ${pt(at(r, a0))} A ${r} ${r} 0 0 1 ${pt(at(r, a1))} Z`;
}
const partMid = (n: Node, i: number, r: number) => at(r, (partAngles(n, i)[0] + partAngles(n, i)[1]) / 2);

/** One line per prerequisite: `d` runs from `from` (null: the core) to `to`, `back` the other way. */
type Edge = { key: string; from: Node | null; to: Node; d: string; back: string; color: string };
function edgePath(from: Node | null, to: Node): string {
  if (!from) {
    const a = (Math.atan2(to.y, to.x) * 180) / Math.PI;
    return `M ${pt(at(CORE_R - 4, a))} L ${f1(to.x)} ${f1(to.y)}`;
  }
  if (from.ring && to.ring) return ringArc(from, to);
  return `M ${f1(from.x)} ${f1(from.y)} L ${f1(to.x)} ${f1(to.y)}`;
}
/** An arc along Serenity's loop. */
function ringArc(a: Node, b: Node): string {
  const [cx, cy] = world(0, [LOOP[0], LOOP[1]]);
  const cross = (a.x - cx) * (b.y - cy) - (a.y - cy) * (b.x - cx);
  return `M ${f1(a.x)} ${f1(a.y)} A ${LOOP[2]} ${LOOP[2]} 0 0 ${cross > 0 ? 1 : 0} ${f1(b.x)} ${f1(b.y)}`;
}
function buildEdges(): Edge[] {
  const out: Edge[] = [];
  const seen = new Set<string>();
  for (const n of NODES) {
    if (n.kind === 'gate') continue;
    if (n.req.length === 0) out.push({ key: `core>${n.id}`, from: null, to: n, d: edgePath(null, n), back: '', color: color(n.sector) });
    for (const pid of n.req) {
      const p = BY_ID.get(pid)!;
      // A link that is a prerequisite both ways is one line.
      const pair = [pid, n.id].sort().join('|');
      if (seen.has(pair)) continue;
      seen.add(pair);
      out.push({ key: `${pid}>${n.id}`, from: p, to: n, d: edgePath(p, n), back: edgePath(n, p), color: color(p.sector) });
    }
  }
  return out;
}
const EDGES = buildEdges();
/**
 * Lit: a prerequisite at full rank (or the core) and the node it opens owned. A link that is a
 * prerequisite both ways (Serenity's loop, Vigilance's rungs) lights from either end; the
 * path to draw runs the way it lit, so its pulse flows into the node it opened.
 */
function edgeLit(e: Edge): string | null {
  if (!e.from) return owned(e.to) ? e.d : null;
  const way = (a: Node, b: Node) => b.req.includes(a.id) && full(a) && owned(b);
  return way(e.from, e.to) ? e.d : way(e.to, e.from) ? e.back : null;
}
/** Whether an edge joins `n` and its prerequisite `p` (either way round: a loop or rung link may be stored from `n`). */
const joins = (e: Edge, n: Node, p: string) => (e.to === n && e.from?.id === p) || (e.from === n && e.to.id === p);

// ---- the frame and the decoration outside it, drawn once (055) ----
/** A small seeded generator, so the decoration is the same on every load and screenshot. */
function rng(seed: number): () => number {
  let s = seed >>> 0;
  return () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let t = s;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const dot = ([x, y]: [number, number], r: number) => `M ${f1(x - r)} ${f1(y)} a ${r} ${r} 0 1 0 ${f1(2 * r)} 0 a ${r} ${r} 0 1 0 ${f1(-2 * r)} 0 `;
/** The bezel's tick marks: fine ones every 2 degrees on its outer edge, longer every 10, and a fine row on its inner edge. */
function bezelTicks(): { minor: string; major: string; inner: string } {
  let [minor, major, inner] = ['', '', ''];
  for (let k = 0; k < 360; k += 2) {
    if (k % 10 === 0) major += `M ${pt(at(BEZEL_OUT - 12, k))} L ${pt(at(BEZEL_OUT - 2, k))} `;
    else minor += `M ${pt(at(BEZEL_OUT - 7, k))} L ${pt(at(BEZEL_OUT - 2, k))} `;
    if (k % 4 === 0) inner += `M ${pt(at(BEZEL_IN + 2, k))} L ${pt(at(BEZEL_IN + 6, k))} `;
  }
  return { minor, major, inner };
}
/**
 * The corona: short dendrite-like filaments growing out of the bezel, each in the color of the
 * sector it leaves. Every step and twig heads within 80 degrees of its root's outward line, so
 * each point lies further out than CORONA_IN along that line: nothing reaches inside the bezel.
 */
function coronaPaths(): { sector: string; d: string; tips: string }[] {
  const r = rng(55);
  const out = SECTORS.map((s) => ({ sector: s.id, d: '', tips: '' }));
  for (let k = 0; k < CORONA_COUNT; k++) {
    const a = (360 * (k + r() * 0.8)) / CORONA_COUNT;
    const i = Math.floor((((a + 90 + SPAN / 2) % 360) + 360) % 360 / SPAN);
    const o = out[i];
    const len = CORONA_LEN[0] + r() * (CORONA_LEN[1] - CORONA_LEN[0]);
    let p = at(CORONA_IN, a);
    let dir = a + (r() - 0.5) * 20;
    o.d += `M ${pt(p)}`;
    for (let s = 0; s < 3; s++) {
      dir = Math.max(a - 40, Math.min(a + 40, dir + (r() - 0.5) * 30));
      p = [p[0] + (len / 3) * Math.cos(rad(dir)), p[1] + (len / 3) * Math.sin(rad(dir))];
      o.d += ` L ${pt(p)}`;
      if (s < 2 && r() < (s === 0 ? 0.3 : 0.6)) {
        const td = dir + (r() < 0.5 ? -1 : 1) * (25 + r() * 15);
        const tl = len * (s === 0 ? 0.45 : 0.32);
        const tp: [number, number] = [p[0] + tl * Math.cos(rad(td)), p[1] + tl * Math.sin(rad(td))];
        o.d += ` M ${pt(p)} L ${pt(tp)} M ${pt(p)}`;
      }
    }
    if (r() < 0.35) o.tips += dot(p, 1.6);
  }
  return out;
}
/**
 * The background: sparse neurons (a soma and a few branching dendrites) with synapse traces
 * between near neighbors, all outside DECO_MIN_R. Somas sit at least 70 further out and a
 * dendrite reaches at most 60, and a trace is kept only if every sampled point clears it.
 */
function neuralField(): { somas: string; dendrites: string; traces: string; boutons: string; pulses: string[] } {
  const r = rng(553);
  const somas: [number, number][] = [];
  for (let tries = 0; somas.length < 90 && tries < 6000; tries++) {
    const p: [number, number] = [(r() * 2 - 1) * 1700, (r() * 2 - 1) * 1000];
    if (Math.hypot(p[0], p[1]) < DECO_MIN_R + 70) continue;
    if (somas.some((q) => Math.hypot(q[0] - p[0], q[1] - p[1]) < 165)) continue;
    somas.push(p);
  }
  let [soma, dendrites, traces, boutons] = ['', '', '', ''];
  for (const p of somas) {
    soma += dot(p, 2.6 + r() * 2.4);
    const arms = 3 + Math.floor(r() * 3);
    for (let k = 0; k < arms; k++) {
      let dir = (360 * (k + r() * 0.7)) / arms;
      let q = p;
      dendrites += `M ${pt(q)}`;
      for (let s = 0; s < 3; s++) {
        dir += (r() - 0.5) * 50;
        q = [q[0] + 13 * Math.cos(rad(dir)), q[1] + 13 * Math.sin(rad(dir))];
        dendrites += ` L ${pt(q)}`;
        if (s === 1 && r() < 0.45) {
          const td = dir + (r() < 0.5 ? -1 : 1) * 40;
          dendrites += ` M ${pt(q)} L ${pt([q[0] + 9 * Math.cos(rad(td)), q[1] + 9 * Math.sin(rad(td))])} M ${pt(q)}`;
        }
      }
    }
  }
  const pulses: string[] = [];
  const seen = new Set<string>();
  somas.forEach((p, i) => {
    const near = somas
      .map((q, j) => ({ j, d: Math.hypot(q[0] - p[0], q[1] - p[1]) }))
      .filter((e) => e.j !== i && e.d < 330)
      .sort((x, y) => x.d - y.d)
      .slice(0, 2);
    for (const { j, d } of near) {
      const key = [i, j].sort((x, y) => x - y).join('-');
      if (seen.has(key)) continue;
      seen.add(key);
      const q = somas[j];
      const bend = (r() - 0.5) * 0.5 * d;
      const c: [number, number] = [(p[0] + q[0]) / 2 - ((q[1] - p[1]) / d) * bend, (p[1] + q[1]) / 2 + ((q[0] - p[0]) / d) * bend];
      const on = (t: number): [number, number] => [
        (1 - t) * (1 - t) * p[0] + 2 * t * (1 - t) * c[0] + t * t * q[0],
        (1 - t) * (1 - t) * p[1] + 2 * t * (1 - t) * c[1] + t * t * q[1],
      ];
      if (Array.from({ length: 21 }, (_, k) => on(k / 20)).some(([x, y]) => Math.hypot(x, y) < DECO_MIN_R)) continue;
      const path = `M ${pt(p)} Q ${pt(c)} ${pt(q)} `;
      traces += path;
      boutons += dot(on(0.9), 2);
      if (seen.size % 4 === 0) pulses.push(path);
    }
  });
  return { somas: soma, dendrites, traces, boutons, pulses };
}
const TICKS = bezelTicks();
const CORONA = coronaPaths();
const FIELD = neuralField();

// ---- the debug toggle and the takeover ----

/** The debug-menu toggle that shows the Cortex mock's button in the left column. */
export function CortexMockToggle() {
  const [on, setOn] = useState(mock.enabled);
  return (
    <p>
      <label>
        <input
          type="checkbox"
          id="cortex-mock-toggle"
          checked={on}
          onChange={(e) => {
            mock.enabled = (e.target as HTMLInputElement).checked;
            setOn(mock.enabled);
            takeoversChanged();
          }}
        />{' '}
        Cortex mock in the left column
      </label>{' '}
      <span class="muted small">the talent tree, UI only: placeholder powers with no effect, lost on reload</span>
    </p>
  );
}

export const cortexMockTakeover: Takeover = {
  id: 'cortex',
  label: 'Cortex (mock)',
  title: (
    <>
      The Cortex <span class="draft-tag">MOCK</span>
    </>
  ),
  bleed: true,
  shown: () => mock.enabled,
  render: () => <CortexMock />,
};

type Hover = { id: string; part: number | null };
/** A hovered locked node's prerequisite: at full rank, or still missing. */
type Need = 'have' | 'miss';

function CortexMock() {
  const [, setTick] = useState(0);
  // Sectors a hovered Theorem lights, and the node under the pointer: the view's own state,
  // so closing it mid-hover leaves nothing lit.
  const [lit, setLit] = useState<string[]>([]);
  const [hover, setHover] = useState<Hover | null>(null);
  const [refusal, setRefusal] = useState<{ id: string; text: string } | null>(null);
  const treeRef = useRef<HTMLDivElement>(null);
  const tipRef = useRef<HTMLDivElement>(null);
  const pointer = useRef<[number, number] | null>(null);
  const redraw = () => setTick((x) => x + 1);
  const p = points(mock.insight);
  const tpFree = p.earned - spentPoints();
  const axEarned = axiomSources(mock.ranks).length;
  const axFree = axEarned - axiomsSpent();

  const add = (n: Node, part: number | null) => {
    setRefusal(null);
    // An owned split socket (or Rhetoric's fan) switches to a dim part in one click, free: it
    // stays owned whichever part is lit, so nothing beyond it is stranded.
    if (n.parts && part !== null && owned(n)) {
      if (mock.picks[n.id] === part) return;
      const from = bare(n.parts[mock.picks[n.id]].name);
      mock.picks[n.id] = part;
      mock.flash = `Switched ${from} to ${bare(n.parts[part].name)}`;
      return redraw();
    }
    const st = stateOf(n, tpFree);
    if (!(st === 'buyable' || (st === 'partial' && tpFree >= 1))) return;
    if (n.parts) {
      if (part === null) return;
      mock.picks[n.id] = part;
    }
    mock.ranks[n.id] = rank(n.id) + 1;
    const nm = bare(n.parts ? n.parts[part!].name : n.name);
    mock.flash = n.kind === 'end' && full(n) ? `+1 Axiom from ${sectorOf(n.sector).name}` : n.parts ? `Took ${nm}` : `Bought ${nm}${n.ranks > 1 ? ` (${rank(n.id)}/${n.ranks})` : ''}`;
    redraw();
  };
  const remove = (n: Node) => {
    if (!owned(n)) return;
    const block = removeBlock(n);
    if (block) {
      setRefusal({ id: n.id, text: block });
      mock.flash = block;
      return redraw();
    }
    setRefusal(null);
    const nm = bare(n.parts ? n.parts[mock.picks[n.id]].name : n.name);
    mock.ranks[n.id] = rank(n.id) - 1;
    if (mock.ranks[n.id] === 0) {
      delete mock.ranks[n.id];
      delete mock.picks[n.id];
    }
    mock.flash = `Refunded a point from ${nm}`;
    redraw();
  };
  const learn = (t: Theorem) => {
    if (mock.theorems.has(t.id) || axFree < t.cost) return;
    mock.theorems.add(t.id);
    mock.flash = `Learned ${bare(t.name)}`;
    redraw();
  };
  const respec = () => {
    mock.ranks = {};
    mock.picks = {};
    mock.theorems.clear();
    mock.flash = 'Respec: every talent point and Axiom refunded (gates stay: the story granted them)';
    redraw();
  };
  const toggleGate = (n: Node) => {
    if (mock.gates.has(n.id)) {
      mock.gates.delete(n.id);
      // Relocking a gate takes its Axiom back; if that leaves Theorems unpaid, all are unlearned.
      if (axiomSources(mock.ranks).length < axiomsSpent()) mock.theorems.clear();
      mock.flash = `${bare(n.name)} relocked`;
    } else {
      mock.gates.add(n.id);
      mock.flash = `Story beat ${n.beat}: ${bare(n.name)} unlocked, +1 Axiom`;
    }
    redraw();
  };
  const enter = (id: string, part: number | null) => {
    if (refusal && refusal.id !== id) setRefusal(null);
    setHover({ id, part });
  };
  const leave = () => {
    setHover(null);
    setRefusal(null);
  };
  // The tooltip follows the pointer without re-rendering the tree; after a render (a new
  // node, or longer text) it is placed again, now that its size is known. It takes the place
  // nearest the pointer, outward (away from the core) first, that fits and covers neither the
  // pointer nor any of a hovered locked node's outlined prerequisites: beside the pointer, or
  // just past the outlined nodes when they crowd it.
  const place = () => {
    const tree = treeRef.current;
    const tip = tipRef.current;
    if (!tree || !tip || !pointer.current) return;
    const b = tree.getBoundingClientRect();
    const core = tree.querySelector('[data-testid="cortex-core"]')!.getBoundingClientRect();
    const [mx, my] = [pointer.current[0] - b.left, pointer.current[1] - b.top];
    const [cx, cy] = [core.left + core.width / 2 - b.left, core.top + core.height / 2 - b.top];
    const [w, h] = [tip.offsetWidth, tip.offsetHeight];
    const keep = [...tree.querySelectorAll('.cx-prereq')].map((el) => {
      const r = el.getBoundingClientRect();
      return { l: r.left - b.left, r: r.right - b.left, t: r.top - b.top, b: r.bottom - b.top };
    });
    keep.push({ l: mx - 12, r: mx + 12, t: my - 12, b: my + 12 });
    const xs = [mx + 18, mx - 18 - w, Math.max(...keep.map((k) => k.r)) + 8, Math.min(...keep.map((k) => k.l)) - 8 - w];
    const ys = [my + 14, my - 14 - h, Math.max(...keep.map((k) => k.b)) + 8, Math.min(...keep.map((k) => k.t)) - 8 - h];
    const fits = ([x, y]: number[]) => x >= 4 && y >= 4 && x + w <= b.width - 4 && y + h <= b.height - 4;
    const clear = ([x, y]: number[]) => keep.every((k) => k.r <= x || k.l >= x + w || k.b <= y || k.t >= y + h);
    // Distance from the pointer to the tooltip's nearest point, plus a step for each inward side.
    const score = ([x, y]: number[]) =>
      Math.hypot(Math.max(x - mx, 0, mx - x - w), Math.max(y - my, 0, my - y - h)) +
      ((x + w / 2 > mx) !== (mx >= cx) ? 30 : 0) +
      ((y + h / 2 > my) !== (my >= cy) ? 30 : 0);
    const all = xs.flatMap((x) => ys.map((y) => [x, y])).sort((p, q) => score(p) - score(q));
    const [x, y] = all.find((c) => fits(c) && clear(c)) ?? all.find(fits) ?? [mx + 18, my + 14];
    tip.style.transform = `translate(${Math.round(Math.max(4, Math.min(x, b.width - w - 4)))}px, ${Math.round(Math.max(4, Math.min(y, b.height - h - 4)))}px)`;
  };
  const onMove = (e: MouseEvent) => {
    pointer.current = [e.clientX, e.clientY];
    place();
  };
  useLayoutEffect(place);
  // The wheel scales to fit its box; `--cx-scale` (screen px per wheel unit) keeps the
  // vignette's clear middle just outside the corona at any window size.
  useLayoutEffect(() => {
    const tree = treeRef.current;
    if (!tree) return;
    const main = tree.parentElement!;
    const fit = () => main.style.setProperty('--cx-scale', String(Math.min(tree.clientWidth, tree.clientHeight) / (2 * VIEW)));
    fit();
    const ro = new ResizeObserver(fit);
    ro.observe(tree);
    return () => ro.disconnect();
  }, []);
  /** Keyboard focus: the tooltip goes by the node, not the last pointer place. */
  const focusAt = (el: Element) => {
    const r = el.getBoundingClientRect();
    pointer.current = [r.right, r.top + r.height / 2];
  };
  const gates = NODES.filter((n) => n.kind === 'gate');
  const hn = hover ? BY_ID.get(hover.id)! : null;
  // A hovered locked node shows why: each prerequisite outlined, `have` at full rank, `miss` not.
  const needLock = hn && stateOf(hn, tpFree) === 'locked' ? hn : null;
  const need: Record<string, Need> = {};
  for (const p of needLock?.req ?? []) need[p] = fullIn(mock.ranks, p) ? 'have' : 'miss';
  return (
    <div class="cx-root" data-testid="cortex-mock">
      <style>{CSS}</style>
      <div class="cx-bar">
        <span>
          Insight <b id="cortex-insight">{fmt(mock.insight)}</b>
        </span>
        <span>
          Talent points <b id="cortex-tp">{tpFree}</b> <span class="cx-dim">free of {p.earned}</span>
        </span>
        <span>
          Next <b id="cortex-next">{fmt(p.toNext)}</b>{' '}
          <span class="cx-track">
            <span class="cx-fill" style={{ width: `${(100 * (p.nextCost - p.toNext)) / p.nextCost}%` }} />
          </span>
        </span>
        <span>
          Axioms <b id="cortex-axioms">{axFree}</b> <span class="cx-dim">free of {axEarned}</span>
        </span>
        <span class="cx-actions">
          {GRANTS.map((g) => (
            <button type="button" key={g} id={`cortex-grant-${g}`} class="cx-btn" onClick={() => ((mock.insight += g), redraw())}>
              +{fmt(g)}
            </button>
          ))}
          <button type="button" id="cortex-respec" class="cx-btn cx-danger" onClick={respec}>
            Respec
          </button>
        </span>
        <span class="cx-gates">
          <span class="cx-dim">Debug, story beats:</span>
          {gates.map((n) => (
            <label key={n.id}>
              <input type="checkbox" data-gate-toggle={n.id} checked={mock.gates.has(n.id)} onChange={() => toggleGate(n)} /> {n.beat}
            </label>
          ))}
        </span>
      </div>
      <div class="cx-main">
        <div class="cx-tree" ref={treeRef} onMouseMove={onMove} onContextMenu={(e) => e.preventDefault()}>
          {/* Decoration outside the bezel only: neurons and synapse traces, fading toward the edges.
              Same viewBox as the wheel, so the two line up; it overflows the square into the box's margins. */}
          <svg class="cx-field" viewBox={`${-VIEW} ${-VIEW} ${2 * VIEW} ${2 * VIEW}`} aria-hidden="true" data-deco="field">
            <path class="cx-field-trace" d={FIELD.traces} />
            {FIELD.pulses.map((d, k) => (
              <path key={k} class="cx-field-pulse" d={d} style={{ animationDelay: `${-1.7 * k}s` }} />
            ))}
            <path class="cx-field-dendrite" d={FIELD.dendrites} />
            <path class="cx-field-soma" d={FIELD.somas} />
            <path class="cx-field-bouton" d={FIELD.boutons} />
          </svg>
          <svg class="cx-svg" viewBox={`${-VIEW} ${-VIEW} ${2 * VIEW} ${2 * VIEW}`} role="group" aria-label="The Cortex tree">
            <defs>
              <radialGradient id="cx-bezel-fill" gradientUnits="userSpaceOnUse" cx="0" cy="0" r={BEZEL_OUT}>
                <stop offset={BEZEL_IN / BEZEL_OUT} style={{ stopColor: 'var(--cx-bezel-edge)' }} />
                <stop offset={(BEZEL_IN + 8) / BEZEL_OUT} style={{ stopColor: 'var(--cx-bezel-mid)' }} />
                <stop offset={(BEZEL_OUT - 14) / BEZEL_OUT} style={{ stopColor: 'var(--cx-bezel-mid)' }} />
                <stop offset="1" style={{ stopColor: 'var(--cx-bezel-edge)' }} />
              </radialGradient>
              <radialGradient id="cx-core-fill">
                <stop offset="0%" stop-color="#fff3d6" />
                <stop offset="35%" stop-color="#ffc861" />
                <stop offset="75%" stop-color="#b36a00" />
                <stop offset="100%" stop-color="#3a2200" />
              </radialGradient>
              <radialGradient id="cx-core-halo">
                <stop offset="45%" stop-color="#f5a524" stop-opacity="0.5" />
                <stop offset="100%" stop-color="#f5a524" stop-opacity="0" />
              </radialGradient>
              {SECTORS.map((s) => (
                <Fragment key={s.id}>
                  <radialGradient id={`cx-glow-${s.id}`}>
                    <stop offset="40%" style={{ stopColor: color(s.id), stopOpacity: 'var(--cx-glow-a)' }} />
                    <stop offset="100%" style={{ stopColor: color(s.id) }} stop-opacity="0" />
                  </radialGradient>
                  <radialGradient key={`f-${s.id}`} id={`cx-fill-${s.id}`} cx="40%" cy="35%">
                    <stop offset="0%" style={{ stopColor: `color-mix(in srgb, ${color(s.id)} var(--cx-hi-mix), #ffffff)` }} />
                    <stop offset="45%" style={{ stopColor: color(s.id) }} />
                    <stop offset="100%" style={{ stopColor: color(s.id), stopOpacity: 'var(--cx-fill-edge-a)' }} />
                  </radialGradient>
                </Fragment>
              ))}
            </defs>
            {/* Sector floors and the rim that a hovered Theorem lights. No boundary line: inside the
                bezel every drawn line is a prerequisite (the floors' tints tell the sectors apart). */}
            {SECTORS.map((s, i) => {
              const [a0, a1] = [center(i) - SPAN / 2, center(i) + SPAN / 2];
              const on = lit.includes(s.id);
              return (
                <g key={s.id} class={`cx-sector${on ? ' cx-lit' : ''}`} data-sector={s.id} data-lit={on ? '1' : '0'} style={{ color: color(s.id) }}>
                  <path class="cx-floor" d={`M ${pt(at(CORE_R + 8, a0))} L ${pt(at(RIM_R, a0))} A ${RIM_R} ${RIM_R} 0 0 1 ${pt(at(RIM_R, a1))} L ${pt(at(CORE_R + 8, a1))} A ${CORE_R + 8} ${CORE_R + 8} 0 0 0 ${pt(at(CORE_R + 8, a0))} Z`} />
                  <path class="cx-rim-glow" d={`M ${pt(at(RIM_R, a0 + 1.2))} A ${RIM_R} ${RIM_R} 0 0 1 ${pt(at(RIM_R, a1 - 1.2))}`} />
                  <path class="cx-rim" d={`M ${pt(at(RIM_R, a0 + 1.2))} A ${RIM_R} ${RIM_R} 0 0 1 ${pt(at(RIM_R, a1 - 1.2))}`} />
                </g>
              );
            })}
            {/* The dial bezel: a brass ring hugging the rim, fine ticks on its edges, a stud on each
                sector boundary; the sector names are engraved on it. */}
            <g class="cx-bezel-ring" data-deco="bezel" aria-hidden="true">
              <circle class="cx-bz-band" r={(BEZEL_IN + BEZEL_OUT) / 2} style={{ strokeWidth: BEZEL_OUT - BEZEL_IN }} />
              <circle class="cx-bz-edge" r={BEZEL_IN} />
              <circle class="cx-bz-edge" r={BEZEL_OUT} />
              <circle class="cx-bz-fine" r={BEZEL_OUT - 13.5} />
              <path class="cx-bz-tick" d={TICKS.minor} />
              <path class="cx-bz-tick cx-bz-major" d={TICKS.major} />
              <path class="cx-bz-tick" d={TICKS.inner} />
              {SECTORS.map((s, i) => {
                const [x, y] = at((BEZEL_IN + BEZEL_OUT) / 2 - 2, center(i) - SPAN / 2);
                return <path key={s.id} class="cx-bz-stud" d={`M ${f1(x)} ${f1(y - 6)} L ${f1(x + 6)} ${f1(y)} L ${f1(x)} ${f1(y + 6)} L ${f1(x - 6)} ${f1(y)} Z`} />;
              })}
            </g>
            {/* The corona: faint filaments growing out of the bezel, in their sector's color. */}
            <g class="cx-corona" data-deco="corona" aria-hidden="true">
              {CORONA.map((c) => (
                <g key={c.sector} class={`cx-corona-sector${lit.includes(c.sector) ? ' cx-lit' : ''}`} style={{ color: color(c.sector) }}>
                  <path class="cx-corona-fil" d={c.d} />
                  <path class="cx-corona-tip" d={c.tips} />
                </g>
              ))}
            </g>
            {/* Sector names along the outer rim; the lower ones run the other way to read upright. */}
            {SECTORS.map((s, i) => {
              const [a0, a1] = [center(i) - SPAN / 2 + 3, center(i) + SPAN / 2 - 3];
              const c = ((center(i) % 360) + 360) % 360;
              const flip = c > 0 && c < 180;
              const r = flip ? NAME_R + 15 : NAME_R;
              const d = flip ? `M ${pt(at(r, a1))} A ${r} ${r} 0 0 0 ${pt(at(r, a0))}` : `M ${pt(at(r, a0))} A ${r} ${r} 0 0 1 ${pt(at(r, a1))}`;
              return (
                <g key={s.id}>
                  <path id={`cx-name-path-${s.id}`} d={d} fill="none" />
                  <text class="cx-sector-name" style={{ fill: color(s.id) }} data-sector-name={s.id}>
                    <textPath href={`#cx-name-path-${s.id}`} startOffset="50%">
                      {s.name.toUpperCase()}
                    </textPath>
                  </text>
                </g>
              );
            })}
            {/* Links, one per prerequisite: dim traces, or glowing outward from the core once bought. */}
            {EDGES.map((e) => {
              const lit = edgeLit(e);
              const on = lit !== null;
              const d = lit ?? e.d;
              return (
                <g
                  key={e.key}
                  class={`cx-edge${on ? ' cx-on' : ''}${e.to.kind === 'bridge' ? ' cx-feed' : ''}`}
                  style={{ color: e.color }}
                  data-edge={e.key}
                  data-from={e.from?.id ?? 'core'}
                  data-to={e.to.id}
                >
                  {on && <path class="cx-e-glow" d={d} />}
                  <path class="cx-e-line" d={e.d} />
                  {on && <path class="cx-e-pulse" d={d} />}
                </g>
              );
            })}
            {/* A hovered locked node's way in: the lines from its prerequisites, cased so they read
                apart from any trace, solid from one at full rank, dotted from one still missing. */}
            {needLock &&
              EDGES.filter((e) => needLock.req.some((p) => joins(e, needLock, p))).map((e) => {
                const p = e.to === needLock ? e.from!.id : e.to.id;
                const d = e.to === needLock ? e.d : e.back;
                return (
                  <g key={`need-${e.key}`} class={`cx-need-line cx-need-${need[p]}`} data-need-edge={e.key} data-need={need[p]}>
                    <path class="cx-e-case" d={d} />
                    <path class="cx-e-need" d={d} />
                  </g>
                );
              })}
            {/* The core: free points. */}
            <g class="cx-core" data-testid="cortex-core">
              <circle class="cx-core-halo" r={CORE_R + 34} />
              <circle class="cx-core-ring" r={CORE_R + 4} />
              <circle class="cx-core-orb" r={CORE_R - 6} />
              <text class="cx-core-num" y="10" id="cortex-core-tp">
                {tpFree}
              </text>
              <text class="cx-core-sub" y="30">
                FREE POINTS
              </text>
              <text class="cx-core-ax" y="-30">
                ◆ {axFree}
              </text>
            </g>
            {NODES.map((n) => (
              <Socket
                key={n.id}
                n={n}
                st={stateOf(n, tpFree)}
                hover={hover?.id === n.id ? hover.part : undefined}
                refused={refusal?.id === n.id}
                need={need[n.id]}
                onAdd={(part) => add(n, part)}
                onRemove={() => remove(n)}
                onEnter={(part) => enter(n.id, part)}
                onFocusAt={focusAt}
                onLeave={leave}
              />
            ))}
          </svg>
          <div class={`cx-tip${hn ? '' : ' cx-hidden'}`} ref={tipRef} data-testid="cortex-tip" role="tooltip">
            {hn && <Tip n={hn} part={hover!.part} st={stateOf(hn, tpFree)} tpFree={tpFree} refusal={refusal?.id === hn.id ? refusal.text : null} />}
          </div>
          <p class="cx-flash" id="cortex-flash">
            {mock.flash ?? 'Left-click adds a point, right-click takes one back. Placeholder powers with no effect; lost on reload.'}
          </p>
        </div>
        <aside class="cx-side">
          {/* A brass display case: a frame with corner studs, a title plate on its top edge, and
              the plaques in a recessed well. */}
          <div class="cx-case" data-testid="cortex-theorems">
            <span class="cx-corner cx-corner-tl" />
            <span class="cx-corner cx-corner-tr" />
            <span class="cx-corner cx-corner-bl" />
            <span class="cx-corner cx-corner-br" />
            <div class="cx-case-plate">
              <h3 class="cx-case-title">Grand Theorems</h3>
            </div>
            <div class="cx-case-well">
              {THEOREMS.map((t) => {
                const have = mock.theorems.has(t.id);
                const unified = t.id === 'unified';
                const can = !have && axFree >= t.cost;
                const light = (ids: string[]) => () => setLit(ids);
                return (
                  <button
                    type="button"
                    key={t.id}
                    class={`cx-plaque${have ? ' cx-have' : ''}${can ? ' cx-can' : ''}${unified ? ' cx-unified' : ''}`}
                    data-theorem={t.id}
                    data-learned={have ? '1' : '0'}
                    aria-disabled={!can}
                    onMouseEnter={light(t.branches)}
                    onMouseLeave={light([])}
                    onFocus={light(t.branches)}
                    onBlur={light([])}
                    onClick={() => learn(t)}
                  >
                    <span class="cx-rivet cx-rivet-l" />
                    <span class="cx-rivet cx-rivet-r" />
                    <span class="cx-plaque-studs">
                      {t.branches.map((b) => (
                        <span key={b} class="cx-stud" style={{ background: color(b) }} title={sectorOf(b).name} />
                      ))}
                    </span>
                    <span class="cx-plaque-name">{bare(t.name)}</span>
                    <span class="cx-plaque-text">{t.text}</span>
                    <span class="cx-plaque-cost">
                      {have ? 'Learned' : <>{'◆'.repeat(t.cost)} <span class="cx-dim">{can ? 'learn' : `${t.cost} Axioms`}</span></>}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>
        </aside>
      </div>
    </div>
  );
}

const glyph = (key: string, size: number, x = 0, y = 0) => (
  <g class="cx-glyph" transform={`translate(${f1(x - size / 2)} ${f1(y - size / 2)}) scale(${(size / 24).toFixed(3)})`} dangerouslySetInnerHTML={{ __html: GLYPHS[key] ?? '' }} />
);

/** One node: a round socket with a glyph, rank pips on the rim; a split socket's parts as slices. */
function Socket({
  n,
  st,
  hover,
  refused,
  need,
  onAdd,
  onRemove,
  onEnter,
  onFocusAt,
  onLeave,
}: {
  n: Node;
  st: State;
  hover: number | null | undefined;
  refused: boolean;
  /** Set when it is a prerequisite of the hovered locked node. */
  need: Need | undefined;
  onAdd: (part: number | null) => void;
  onRemove: () => void;
  onEnter: (part: number | null) => void;
  onFocusAt: (el: Element) => void;
  onLeave: () => void;
}) {
  const c = color(n.sector);
  const r = n.size;
  const pick = mock.picks[n.id];
  const ctx = (e: MouseEvent) => {
    e.preventDefault();
    onRemove();
  };
  const handlers = (part: number | null) => ({
    onMouseEnter: () => onEnter(part),
    onClick: (e: MouseEvent) => {
      e.stopPropagation();
      onAdd(part);
    },
  });
  let body;
  if (n.parts) {
    body = (
      <>
        <circle class="cx-bezel" r={r + 3} />
        {n.parts.map((pp, i) => {
          const taken = pick === i;
          const [gx, gy] = partMid(n, i, r * (n.parts!.length === 3 ? 0.5 : 0.48));
          const [ox, oy] = partMid(n, i, 1.6);
          return (
            <g
              key={i}
              class={`cx-part${taken ? ' cx-taken' : pick !== undefined ? ' cx-other' : ''}${hover === i ? ' cx-hov' : ''}`}
              data-part={i}
              transform={`translate(${f1(ox)} ${f1(oy)})`}
              {...handlers(i)}
            >
              <path class="cx-slice" d={slice(n, i, r)} style={taken ? { fill: `url(#cx-fill-${n.sector})` } : undefined} />
              {glyph(pp.glyph, n.parts!.length === 3 ? 15 : 14, gx, gy)}
            </g>
          );
        })}
      </>
    );
  } else if (n.kind === 'bridge') {
    const next = nextSector(n.sector).id;
    const other = color(next);
    // Each half of the bezel in one neighbor's color, split across the boundary; once bought,
    // each half of the disc is filled like a bought node of that neighbor.
    const a = center(sectorIndex(n.sector)) + SPAN / 2;
    const arc = (from: number, to: number) => `M ${pt(at(r + 2, from))} A ${r + 2} ${r + 2} 0 0 1 ${pt(at(r + 2, to))}`;
    const half = (from: number, to: number) => `M ${pt(at(r, from))} A ${r} ${r} 0 0 1 ${pt(at(r, to))} Z`;
    body = (
      <g {...handlers(null)}>
        <circle class="cx-disc" r={r} />
        {st === 'owned' && (
          <>
            <path class="cx-bridge-fill" data-half={n.sector} d={half(a + 180, a + 360)} style={{ fill: `url(#cx-fill-${n.sector})` }} />
            <path class="cx-bridge-fill" data-half={next} d={half(a, a + 180)} style={{ fill: `url(#cx-fill-${next})` }} />
          </>
        )}
        <path class="cx-half" d={arc(a + 180, a + 360)} style={{ stroke: c }} />
        <path class="cx-half" d={arc(a, a + 180)} style={{ stroke: other }} />
        {glyph(n.glyph, r * 1.05)}
      </g>
    );
  } else {
    body = (
      <g {...handlers(null)}>
        {n.kind === 'end' && <circle class="cx-end-ring" r={r + 6} />}
        <circle class="cx-bezel" r={r + 2.5} />
        <circle class="cx-disc" r={r} style={st === 'owned' ? { fill: `url(#cx-fill-${n.sector})` } : undefined} />
        {st === 'partial' && <circle class="cx-partial-fill" r={r * (0.35 + (0.55 * rank(n.id)) / n.ranks)} />}
        {glyph(n.kind === 'gate' && st !== 'owned' ? 'lock' : n.glyph, r * (n.kind === 'end' ? 1.1 : 1.15))}
      </g>
    );
  }
  // Rank pips around the top of the rim.
  const pips =
    n.ranks > 1
      ? Array.from({ length: n.ranks }, (_, k) => {
          const a = -90 + (k - (n.ranks - 1) / 2) * 24;
          const [x, y] = at(r + 9, a);
          return <circle key={k} class={`cx-pip${k < rank(n.id) ? ' cx-pip-on' : ''}`} cx={f1(x)} cy={f1(y)} r="3.6" />;
        })
      : null;
  return (
    <g
      class={`cx-node cx-${n.kind} cx-${st}${hover !== undefined ? ' cx-hover' : ''}${refused ? ' cx-refused' : ''}`}
      transform={`translate(${f1(n.x)} ${f1(n.y)})`}
      style={{ color: c }}
      data-node={n.id}
      data-state={st}
      data-rank={rank(n.id)}
      data-pick={pick ?? ''}
      data-req={n.req.join(' ')}
      data-r={r}
      data-need={need ?? ''}
      role="button"
      tabIndex={0}
      aria-label={`${n.name}: ${n.parts ? n.parts.map((pp) => pp.name).join(' / ') : n.text}`}
      onMouseLeave={onLeave}
      onContextMenu={ctx}
      onFocus={(e) => {
        onFocusAt(e.currentTarget as Element);
        onEnter(n.parts ? (pick ?? 0) : null);
      }}
      onBlur={onLeave}
      onKeyDown={(e) => {
        // Kept here: the page's own keys (space pauses) must not see them.
        const k = n.parts?.length ?? 0;
        const cur = typeof hover === 'number' ? hover : 0;
        if (e.key === 'Enter' || e.key === ' ') onAdd(k ? cur : null);
        else if (e.key === 'Delete' || e.key === 'Backspace') onRemove();
        else if (k && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) onEnter((cur + (e.key === 'ArrowRight' ? 1 : k - 1)) % k);
        else return;
        e.preventDefault();
        e.stopPropagation();
      }}
    >
      {(st === 'buyable' || st === 'owned' || st === 'partial') && <circle class="cx-halo" r={r + 16} style={{ fill: `url(#cx-glow-${n.sector})` }} />}
      {need && (
        <g class={`cx-prereq cx-need-${need}`}>
          <circle class="cx-e-case" r={r + (n.kind === 'end' ? 11 : n.kind === 'split' ? 8 : 7)} />
          <circle class="cx-e-need" r={r + (n.kind === 'end' ? 11 : n.kind === 'split' ? 8 : 7)} />
        </g>
      )}
      {body}
      {pips}
      {(n.kind === 'end' || n.kind === 'gate') && st === 'owned' && (
        <g class="cx-axiom-badge" transform={`translate(${f1(r * 0.75)} ${f1(-r * 0.75)})`}>
          <path d="M 0 -6 L 6 0 L 0 6 L -6 0 Z" />
        </g>
      )}
    </g>
  );
}

/** The tooltip: name, ranks, effect, cost, and why it's locked or refused. */
function Tip({ n, part, st, tpFree, refusal }: { n: Node; part: number | null; st: State; tpFree: number; refusal: string | null }) {
  const s = sectorOf(n.sector);
  const pp = n.parts && part !== null ? n.parts[part] : null;
  const kind =
    n.kind === 'bridge'
      ? `Bridge · ${s.name} and ${nextSector(n.sector).name}`
      : `${s.name} · ${s.shape} · ${n.kind === 'end' ? 'end node: grants an Axiom' : n.kind === 'gate' ? 'story gate: grants an Axiom' : n.role}`;
  const cost = n.kind === 'gate' ? `Free, from the story beat ${n.beat}` : n.parts ? '1 talent point, one part lit at a time (switching is free)' : n.ranks > 1 ? `1 talent point a rank, ${n.ranks} ranks` : '1 talent point';
  const line = refusal ?? why(n, st, tpFree, part);
  const switchable = !!n.parts && st === 'owned' && part !== null && part !== mock.picks[n.id];
  const tone = refusal ? 'cx-tip-bad' : st === 'buyable' || (st === 'partial' && tpFree >= 1) || switchable ? 'cx-tip-ok' : st === 'owned' ? 'cx-tip-own' : 'cx-tip-muted';
  return (
    <>
      <div class="cx-tip-kind" style={{ color: n.kind === 'bridge' ? undefined : `color-mix(in srgb, ${color(s.id)} var(--cx-ink-mix), var(--cx-strong))` }}>
        {kind}
      </div>
      <div class="cx-tip-name">{bare(pp ? pp.name : n.name)}</div>
      {n.parts && (
        <div class="cx-tip-parts">
          {n.parts.map((q, i) => (
            <span key={i} class={`${i === part ? 'cx-cur' : ''}${mock.picks[n.id] === i ? ' cx-picked' : ''}`}>
              {bare(q.name)}
            </span>
          ))}
        </div>
      )}
      {n.ranks > 1 && (
        <div class="cx-tip-ranks">
          {Array.from({ length: n.ranks }, (_, k) => (
            <span key={k} class={k < rank(n.id) ? 'cx-on' : ''} />
          ))}{' '}
          Rank {rank(n.id)} of {n.ranks}
        </div>
      )}
      <div class="cx-tip-text">{bare(pp ? pp.text : n.text)}</div>
      <div class="cx-tip-cost">Cost: {cost}</div>
      <div class={`cx-tip-why ${tone}`} data-testid="cortex-tip-why">
        {line}
      </div>
      <div class="cx-tip-ph">Placeholder: no effect on the game</div>
    </>
  );
}

// The mock follows the app's theme (Matt, 053): contrast first, on a plain solid ground inside
// the rim; decoration only outside the bezel (Matt, 055). Every color is a `--cx-*` token: the light block is the default (as the app's
// is), the dark block overrides it; sector colors are tokens too, re-tuned for each ground.
const CSS = `
.cx-root {
  --cx-bg: #efede8; --cx-text: #211f1b; --cx-dim: #5f5a52; --cx-strong: #14120f;
  --cx-bar-bg: #e4e1da; --cx-bar-line: #cdc8bd; --cx-num: #8f5600;
  --cx-btn-bg: #faf9f6; --cx-btn-line: #b9b3a8; --cx-accent: #b86e00; --cx-danger: #b4352c; --cx-track: #d6d1c7;
  --cx-serenity: #2f6bc9; --cx-logistics: #b86e00; --cx-logic: #23864a; --cx-rhetoric: #c8433a; --cx-vigilance: #7b4bc0;
  --cx-floor-a: 0.07; --cx-floor-lit-a: 0.16;
  --cx-sock: #fbfaf7; --cx-bezel: #8c867c; --cx-locked-fill: #e7e4de; --cx-locked-bezel: #b5afa4; --cx-locked-a: 0.55;
  --cx-other-fill: #ebe8e2; --cx-part-hov: #f6ead6;
  --cx-edge: #a39d92; --cx-feed-a: 0.55;
  --cx-glow-a: 0.4; --cx-hi-mix: 62%; --cx-fill-edge-a: 0.85; --cx-e-glow-a: 0.18; --cx-pulse: #ffffff;
  --cx-owned-glyph: #ffffff; --cx-owned-ring: #ffffff; --cx-hover: #14120f; --cx-refused: #c62828;
  --cx-flash-bg: rgb(251 250 247 / 92%); --cx-flash-line: #cdc8bd; --cx-flash-text: #7a4800;
  --cx-tip-bg: #fdfcfa; --cx-tip-line: #b9b3a8; --cx-tip-shadow: rgb(0 0 0 / 22%); --cx-tip-text: #2b2823; --cx-tip-sep: #e0dbd2;
  --cx-tip-pick-bg: #8f5600; --cx-tip-pick-text: #ffffff; --cx-ink-mix: 70%;
  --cx-ok: #1f7a42; --cx-own: #8f5600; --cx-muted: #625d55; --cx-bad: #b4352c; --cx-ph: #7d776d;
  --cx-brass-bg: linear-gradient(180deg, #f6ecd6 0%, #ecdfc2 100%); --cx-brass-line: #b89a5e;
  --cx-brass-inset: #f6ecd6; --cx-brass-inner: #d2bb88; --cx-brass-text: #4f4126; --cx-brass-name: #3d2d0e; --cx-brass-sub: #6b5c40;
  --cx-brass-cost: #8f5600; --cx-brass-hover: #a5823f; --cx-brass-can: #a5823f;
  --cx-brass-have-bg: linear-gradient(180deg, #f6d991 0%, #eac36a 100%); --cx-brass-have-line: #8f5600; --cx-brass-have-inset: #eac36a;
  --cx-brass-have-text: #3d2d0e; --cx-rivet: radial-gradient(circle at 35% 35%, #fff4d8, #a5823f); --cx-stud-line: #4f4126;
  --cx-bezel-edge: #d9c393; --cx-bezel-mid: #f2e6c8; --cx-bezel-line: #9c7b3c; --cx-bezel-tick: #8a6a2e; --cx-bezel-stud: #b8924a;
  --cx-corona-a: 0.42; --cx-corona-lit-a: 0.8;
  --cx-field: #8d7d62; --cx-field-a: 0.5; --cx-field-pulse: #b86e00; --cx-vignette: rgb(120 100 70 / 20%);
  --cx-case-frame: linear-gradient(180deg, #ead9ad 0%, #cfb06f 48%, #b8954f 100%); --cx-case-line: #8a6a2e;
  --cx-case-well: linear-gradient(180deg, #e4dece 0%, #ece8de 100%); --cx-case-well-shade: rgb(70 50 15 / 30%);
  --cx-case-shadow: rgb(70 50 15 / 24%); --cx-case-hi: rgb(255 248 228 / 80%);
  --cx-plate-bg: linear-gradient(180deg, #f8ebc6 0%, #e2c88f 100%); --cx-plate-text: #3d2d0e;
}
:root[data-theme='dark'] .cx-root {
  --cx-bg: #15181d; --cx-text: #d8dee6; --cx-dim: #8b97a3; --cx-strong: #f1f4f7;
  --cx-bar-bg: #0f1216; --cx-bar-line: #262e37; --cx-num: #ffd27a;
  --cx-btn-bg: #1d242c; --cx-btn-line: #3a4652; --cx-accent: #f5a524; --cx-danger: #f28b82; --cx-track: #262e37;
  --cx-serenity: #8ab4f8; --cx-logistics: #f5a524; --cx-logic: #62d98b; --cx-rhetoric: #f28b82; --cx-vigilance: #c39bf0;
  --cx-floor-a: 0.045; --cx-floor-lit-a: 0.11;
  --cx-sock: #0c0f13; --cx-bezel: #64717e; --cx-locked-fill: #111419; --cx-locked-bezel: #343e48; --cx-locked-a: 0.45;
  --cx-other-fill: #0e1115; --cx-part-hov: #1f2a36;
  --cx-edge: #4a5662; --cx-feed-a: 0.5;
  --cx-glow-a: 0.55; --cx-hi-mix: 0%; --cx-fill-edge-a: 0.7; --cx-e-glow-a: 0.22; --cx-pulse: #ffffff;
  --cx-owned-glyph: #0a0f15; --cx-owned-ring: #ffffff; --cx-hover: #ffffff; --cx-refused: #ff6b6b;
  --cx-flash-bg: rgb(10 12 15 / 85%); --cx-flash-line: #2f3944; --cx-flash-text: #ffd27a;
  --cx-tip-bg: #1a2028; --cx-tip-line: #3f4c59; --cx-tip-shadow: rgb(0 0 0 / 60%); --cx-tip-text: #d8dee6; --cx-tip-sep: #2c3742;
  --cx-tip-pick-bg: #ffd27a; --cx-tip-pick-text: #0a0f15; --cx-ink-mix: 100%;
  --cx-ok: #7ee2a0; --cx-own: #ffd27a; --cx-muted: #9aa8b5; --cx-bad: #ff8a80; --cx-ph: #75828e;
  --cx-brass-bg: linear-gradient(180deg, #2a2418 0%, #1b1710 55%, #15120c 100%); --cx-brass-line: #5a4a2a;
  --cx-brass-inset: #15120c; --cx-brass-inner: #3d3220; --cx-brass-text: #cfc6b2; --cx-brass-name: #e8d6a6; --cx-brass-sub: #a89e88;
  --cx-brass-cost: #ffd27a; --cx-brass-hover: #8a7040; --cx-brass-can: #c9a45c;
  --cx-brass-have-bg: linear-gradient(180deg, #4a3a18 0%, #2e2410 60%, #1f180b 100%); --cx-brass-have-line: #ffd27a; --cx-brass-have-inset: #1f180b;
  --cx-brass-have-text: #f3e6c4; --cx-rivet: radial-gradient(circle at 35% 35%, #d6c08a, #5a4a2a); --cx-stud-line: #0b1016;
  --cx-bezel-edge: #17130c; --cx-bezel-mid: #2b2416; --cx-bezel-line: #6b5630; --cx-bezel-tick: #a08550; --cx-bezel-stud: #a5823f;
  --cx-corona-a: 0.3; --cx-corona-lit-a: 0.7;
  --cx-field: #7d8d9c; --cx-field-a: 0.26; --cx-field-pulse: #c9a45c; --cx-vignette: rgb(0 0 0 / 42%);
  --cx-case-frame: linear-gradient(180deg, #5c4b2a 0%, #3b3019 48%, #2a2212 100%); --cx-case-line: #7a6236;
  --cx-case-well: linear-gradient(180deg, #0f1216 0%, #13161b 100%); --cx-case-well-shade: rgb(0 0 0 / 60%);
  --cx-case-shadow: rgb(0 0 0 / 55%); --cx-case-hi: rgb(214 192 138 / 30%);
  --cx-plate-bg: linear-gradient(180deg, #6e5a32 0%, #43361d 100%); --cx-plate-text: #f3e2b4;
}
.cx-root { display: flex; flex-direction: column; height: 100%; min-height: 0; color: var(--cx-text); background: var(--cx-bg);
  --cx-name-size: 21px; --cx-name-track: 7px; }
.cx-bar { display: flex; flex-wrap: wrap; align-items: center; gap: 3px 18px; padding: 5px 12px;
  background: var(--cx-bar-bg); border-bottom: 1px solid var(--cx-bar-line); font-size: 13px; }
.cx-bar b { font-family: var(--font-mono); color: var(--cx-num); }
.cx-dim { color: var(--cx-dim); }
.cx-actions { display: flex; gap: 5px; }
.cx-btn { background: var(--cx-btn-bg); color: var(--cx-text); border: 1px solid var(--cx-btn-line); border-radius: 3px; padding: 1px 8px; font-size: 12px; cursor: pointer; }
.cx-btn:hover { border-color: var(--cx-accent); }
.cx-danger { color: var(--cx-danger); }
.cx-gates { display: flex; gap: 10px; align-items: center; margin-left: auto; font-size: 12px; }
.cx-track { display: inline-block; width: 60px; height: 6px; background: var(--cx-track); border-radius: 3px; vertical-align: middle; overflow: hidden; }
.cx-fill { display: block; height: 100%; background: var(--cx-accent); }
/* The Theorem case on the left, the wheel on the right. The vignette darkens toward the edges,
   clear out to just past the corona (the wheel's on-screen scale times a wheel radius). */
.cx-main { flex: 1; min-height: 0; display: grid; --cx-side-w: 312px; grid-template-columns: var(--cx-side-w) minmax(0, 1fr);
  background: radial-gradient(circle farthest-corner at calc(var(--cx-side-w) + (100% - var(--cx-side-w)) / 2) 50%,
    transparent calc(var(--cx-scale, 0.7) * 640px), var(--cx-vignette) calc(var(--cx-scale, 0.7) * 640px + 420px)); }
.cx-tree { position: relative; min-height: 0; overflow: hidden; grid-column: 2; grid-row: 1; }
.cx-svg { position: absolute; inset: 0; width: 100%; height: 100%; display: block; user-select: none; }
.cx-field { position: absolute; inset: 0; width: 100%; height: 100%; display: block; pointer-events: none;
  mask-image: radial-gradient(ellipse farthest-corner at 50% 50%, #000 62%, transparent 100%); }
.cx-field path { fill: none; stroke: var(--cx-field); stroke-opacity: var(--cx-field-a); stroke-linecap: round; stroke-linejoin: round; }
.cx-field-trace { stroke-width: 1; }
.cx-field-dendrite { stroke-width: 1.3; }
.cx-field .cx-field-soma, .cx-field .cx-field-bouton { fill: var(--cx-field); fill-opacity: calc(var(--cx-field-a) * 1.6); stroke: none; }
.cx-field .cx-field-pulse { stroke: var(--cx-field-pulse); stroke-opacity: calc(var(--cx-field-a) * 2); stroke-width: 1.6;
  stroke-dasharray: 6 600; animation: cx-synapse 7s linear infinite; }
@keyframes cx-synapse { from { stroke-dashoffset: 606; } to { stroke-dashoffset: 0; } }

.cx-bz-band { fill: none; stroke: url(#cx-bezel-fill); }
.cx-bz-edge { fill: none; stroke: var(--cx-bezel-line); stroke-width: 1.6; }
.cx-bz-fine { fill: none; stroke: var(--cx-bezel-line); stroke-width: 0.6; stroke-opacity: 0.7; }
.cx-bz-tick { fill: none; stroke: var(--cx-bezel-tick); stroke-width: 0.9; }
.cx-bz-major { stroke-width: 1.5; }
.cx-bz-stud { fill: var(--cx-bezel-stud); stroke: var(--cx-bezel-line); stroke-width: 1; }
.cx-corona-sector { opacity: var(--cx-corona-a); transition: opacity 0.2s; }
.cx-corona-sector.cx-lit { opacity: var(--cx-corona-lit-a); }
.cx-corona { animation: cx-corona 6s ease-in-out infinite alternate; }
@keyframes cx-corona { from { opacity: 0.7; } to { opacity: 1; } }
.cx-corona-fil { fill: none; stroke: currentColor; stroke-width: 1.2; stroke-linecap: round; stroke-linejoin: round; }
.cx-corona-tip { fill: currentColor; stroke: none; }
.cx-flash { position: absolute; left: 12px; bottom: 8px; margin: 0; max-width: 46%; font-size: 12.5px; color: var(--cx-flash-text);
  background: var(--cx-flash-bg); border: 1px solid var(--cx-flash-line); border-radius: 4px; padding: 4px 9px; pointer-events: none; }

.cx-floor { fill: currentColor; fill-opacity: var(--cx-floor-a); stroke: none; transition: fill-opacity 0.2s; }
.cx-rim { fill: none; stroke: currentColor; stroke-opacity: 0.6; stroke-width: 3; stroke-linecap: round; transition: stroke-opacity 0.2s, stroke-width 0.2s; }
.cx-rim-glow { fill: none; stroke: currentColor; stroke-opacity: 0; stroke-width: 18; stroke-linecap: round; transition: stroke-opacity 0.2s; }
.cx-sector.cx-lit .cx-rim { stroke-opacity: 1; stroke-width: 6; }
.cx-sector.cx-lit .cx-rim-glow { stroke-opacity: 0.22; }
.cx-sector.cx-lit .cx-floor { fill-opacity: var(--cx-floor-lit-a); }
.cx-sector-name { font-size: var(--cx-name-size); font-weight: 700; letter-spacing: var(--cx-name-track); text-anchor: middle; }

/* A hovered locked node's prerequisites and the lines from them, cased in the hover ink so no
   trace (a bridge's dashed feed, a sector's own red or green) looks like one: solid at full
   rank, dotted missing. */
.cx-prereq, .cx-need-line { pointer-events: none; }
.cx-e-case { fill: none; stroke: var(--cx-hover); stroke-width: 7.5; stroke-linecap: round; }
.cx-e-need { fill: none; stroke-width: 3.6; stroke-linecap: round; }
.cx-need-have .cx-e-need { stroke: var(--cx-ok); }
.cx-need-miss .cx-e-need { stroke: var(--cx-bad); stroke-dasharray: 0.1 7.5; stroke-width: 4.4; }

.cx-edge { pointer-events: none; }
.cx-e-line { fill: none; stroke: var(--cx-edge); stroke-width: 2.4; stroke-linecap: round; }
.cx-feed .cx-e-line { stroke: currentColor; stroke-opacity: var(--cx-feed-a); stroke-dasharray: 6 5; }
.cx-on .cx-e-line { stroke: currentColor; stroke-opacity: 1; stroke-width: 3; stroke-dasharray: none; }
.cx-e-glow { fill: none; stroke: currentColor; stroke-opacity: var(--cx-e-glow-a); stroke-width: 11; stroke-linecap: round; }
.cx-e-pulse { fill: none; stroke: var(--cx-pulse); stroke-opacity: 0.85; stroke-width: 2.2; stroke-linecap: round;
  stroke-dasharray: 3 47; animation: cx-flow 2.6s linear infinite; }
@keyframes cx-flow { from { stroke-dashoffset: 50; } to { stroke-dashoffset: 0; } }

.cx-core-halo { fill: url(#cx-core-halo); animation: cx-breathe 4s ease-in-out infinite alternate; }
.cx-core-ring { fill: var(--cx-sock); stroke: var(--cx-accent); stroke-width: 2; stroke-dasharray: 2 4; }
.cx-core-orb { fill: url(#cx-core-fill); }
.cx-core-num { font-size: 36px; font-weight: 800; text-anchor: middle; fill: #2a1800; font-family: var(--font-mono); }
.cx-core-sub { font-size: 8.5px; letter-spacing: 1.5px; font-weight: 700; text-anchor: middle; fill: #4a2c00; }
.cx-core-ax { font-size: 12px; font-weight: 700; text-anchor: middle; fill: #4a2c00; font-family: var(--font-mono); }
@keyframes cx-breathe { from { opacity: 0.55; } to { opacity: 1; } }

.cx-node { cursor: pointer; outline: none; }
.cx-bezel { fill: var(--cx-sock); stroke: var(--cx-bezel); stroke-width: 2; }
.cx-disc { fill: var(--cx-sock); stroke: currentColor; stroke-width: 2.2; }
.cx-glyph { fill: currentColor; color: inherit; pointer-events: none; }
.cx-halo { pointer-events: none; }
.cx-buyable .cx-halo { animation: cx-pulse 2.4s ease-in-out infinite alternate; }
@keyframes cx-pulse { from { opacity: 0.25; } to { opacity: 0.95; } }
.cx-owned .cx-halo { opacity: 0.6; }
.cx-partial .cx-halo { opacity: 0.35; }
.cx-locked, .cx-gate-locked { cursor: default; }
.cx-locked .cx-disc, .cx-gate-locked .cx-disc { fill: var(--cx-locked-fill); stroke-opacity: var(--cx-locked-a); stroke-width: 1.4; }
.cx-locked .cx-bezel, .cx-gate-locked .cx-bezel { fill: var(--cx-locked-fill); stroke: var(--cx-locked-bezel); }
.cx-locked .cx-glyph, .cx-gate-locked .cx-glyph { opacity: var(--cx-locked-a); }
.cx-gate .cx-bezel { stroke-dasharray: 4 3; stroke: currentColor; stroke-opacity: 0.7; }
.cx-open .cx-glyph { opacity: 0.75; }
.cx-partial-fill { fill: currentColor; opacity: 0.32; pointer-events: none; }
.cx-owned:not(.cx-split):not(.cx-bridge) .cx-disc { stroke: var(--cx-owned-ring); stroke-opacity: 0.6; }
.cx-owned:not(.cx-split) .cx-glyph { fill: var(--cx-owned-glyph); }
.cx-owned .cx-bezel { stroke: currentColor; }
.cx-end-ring { fill: none; stroke: currentColor; stroke-opacity: 0.55; stroke-width: 1.5; stroke-dasharray: 1 3.2; }
.cx-owned .cx-end-ring { stroke-opacity: 1; stroke-dasharray: none; }
.cx-half { fill: none; stroke-width: 4; stroke-linecap: round; }
.cx-locked .cx-half { stroke-opacity: var(--cx-locked-a); stroke-width: 2.5; }
.cx-bridge-fill { pointer-events: none; }
.cx-pip { fill: var(--cx-sock); stroke: currentColor; stroke-width: 1.4; stroke-opacity: 0.8; pointer-events: none; }
.cx-pip-on { fill: currentColor; stroke: var(--cx-hover); stroke-opacity: 0.8; }
.cx-axiom-badge path { fill: #ffd27a; stroke: #3a2200; stroke-width: 1.2; }
.cx-slice { fill: var(--cx-sock); stroke: currentColor; stroke-width: 1.8; }
.cx-part .cx-glyph { opacity: 0.85; }
.cx-locked .cx-slice { fill: var(--cx-locked-fill); stroke-opacity: var(--cx-locked-a); }
.cx-locked .cx-part .cx-glyph { opacity: var(--cx-locked-a); }
.cx-buyable .cx-part:hover .cx-slice, .cx-buyable .cx-part.cx-hov .cx-slice { fill: var(--cx-part-hov); }
.cx-part.cx-taken .cx-glyph { fill: var(--cx-owned-glyph); opacity: 1; }
.cx-part.cx-other .cx-slice { fill: var(--cx-other-fill); stroke-opacity: 0.45; }
.cx-part.cx-other .cx-glyph { opacity: 0.4; }
.cx-owned .cx-part.cx-other:hover .cx-slice, .cx-owned .cx-part.cx-other.cx-hov .cx-slice { fill: var(--cx-part-hov); stroke-opacity: 1; }
.cx-owned .cx-part.cx-other:hover .cx-glyph, .cx-owned .cx-part.cx-other.cx-hov .cx-glyph { opacity: 0.9; }
.cx-hover .cx-bezel { stroke: var(--cx-hover); stroke-width: 2.4; }
.cx-refused .cx-bezel { stroke: var(--cx-refused); stroke-width: 3; }
.cx-node:focus-visible .cx-bezel { stroke: var(--cx-hover); }
@media (prefers-reduced-motion: reduce) { .cx-e-pulse, .cx-core-halo, .cx-buyable .cx-halo, .cx-corona { animation: none; } .cx-field-pulse { display: none; } }

.cx-tip { position: absolute; left: 0; top: 0; width: 270px; z-index: 5; pointer-events: none;
  background: var(--cx-tip-bg); border: 1px solid var(--cx-tip-line);
  border-radius: 5px; padding: 8px 10px; box-shadow: 0 6px 22px var(--cx-tip-shadow); font-size: 12.5px; display: flex; flex-direction: column; gap: 3px; }
.cx-tip.cx-hidden { display: none; }
.cx-tip-kind { font-size: 11px; text-transform: uppercase; letter-spacing: 0.6px; color: var(--cx-muted); }
.cx-tip-name { font-size: 15px; font-weight: 700; color: var(--cx-strong); }
.cx-tip-parts { display: flex; gap: 4px; flex-wrap: wrap; }
.cx-tip-parts span { font-size: 11px; padding: 0 5px; border: 1px solid var(--cx-tip-line); border-radius: 3px; color: var(--cx-muted); }
.cx-tip-parts .cx-cur { color: var(--cx-strong); border-color: var(--cx-muted); }
.cx-tip-parts .cx-picked { color: var(--cx-tip-pick-text); background: var(--cx-tip-pick-bg); border-color: var(--cx-tip-pick-bg); }
.cx-tip-ranks { display: flex; align-items: center; gap: 4px; color: var(--cx-muted); font-size: 11.5px; }
.cx-tip-ranks span { width: 8px; height: 8px; border-radius: 50%; border: 1.4px solid var(--cx-muted); }
.cx-tip-ranks span.cx-on { background: var(--cx-tip-pick-bg); border-color: var(--cx-tip-pick-bg); }
.cx-tip-text { color: var(--cx-tip-text); }
.cx-tip-cost { color: var(--cx-muted); font-size: 11.5px; }
.cx-tip-why { font-size: 12px; border-top: 1px solid var(--cx-tip-sep); padding-top: 4px; margin-top: 2px; }
.cx-tip-ok { color: var(--cx-ok); }
.cx-tip-own { color: var(--cx-own); }
.cx-tip-muted { color: var(--cx-muted); }
.cx-tip-bad { color: var(--cx-bad); font-weight: 600; }
.cx-tip-ph { font-size: 10.5px; color: var(--cx-ph); font-style: italic; }

/* The left column, clear of the game's nav by a wide left pad. The case is centered on the
   wheel's core (the middle of .cx-main) by auto margins, which fall to zero when it overflows,
   so a short window top-aligns it instead of clipping both ends. */
.cx-side { grid-column: 1; grid-row: 1; display: flex; flex-direction: column; min-height: 0; overflow: hidden; padding: 20px 4px 20px 30px; }
.cx-case { position: relative; display: flex; flex-direction: column; min-height: 0; margin: auto 0; padding: 0 9px 9px;
  background: var(--cx-case-frame); border: 1px solid var(--cx-case-line); border-radius: 7px;
  box-shadow: inset 0 1px 0 var(--cx-case-hi), 0 8px 26px var(--cx-case-shadow); }
.cx-corner { position: absolute; width: 9px; height: 9px; transform: rotate(45deg); background: var(--cx-rivet); border: 1px solid var(--cx-case-line); }
.cx-corner-tl { top: -5px; left: -5px; }
.cx-corner-tr { top: -5px; right: -5px; }
.cx-corner-bl { bottom: -5px; left: -5px; }
.cx-corner-br { bottom: -5px; right: -5px; }
/* The title plate sits on the frame's top edge: a chamfered brass plate with a dark keyline. */
.cx-case-plate { align-self: center; margin: -13px 0 7px; padding: 1.5px; background: var(--cx-case-line);
  clip-path: polygon(11px 0, calc(100% - 11px) 0, 100% 50%, calc(100% - 11px) 100%, 11px 100%, 0 50%); }
.cx-case-title { margin: 0; padding: 5px 22px 5px 26px; background: var(--cx-plate-bg); color: var(--cx-plate-text);
  clip-path: polygon(10px 0, calc(100% - 10px) 0, 100% 50%, calc(100% - 10px) 100%, 10px 100%, 0 50%);
  font-size: 14px; font-weight: 700; line-height: 1.2; letter-spacing: 4px; text-transform: uppercase; white-space: nowrap; }
.cx-case-well { display: flex; flex-direction: column; gap: 7px; min-height: 0; overflow: hidden; padding: 8px;
  background: var(--cx-case-well); border: 1px solid var(--cx-case-line); border-radius: 4px;
  box-shadow: inset 0 2px 10px var(--cx-case-well-shade); }
.cx-plaque { position: relative; flex: 0 1 auto; min-height: 0; display: flex; flex-direction: column; align-items: center; gap: 2px; width: 100%;
  padding: 7px 14px 6px; text-align: center; cursor: default; color: var(--cx-brass-text); background: var(--cx-brass-bg);
  border: 1px solid var(--cx-brass-line); border-radius: 4px; box-shadow: inset 0 0 0 3px var(--cx-brass-inset), inset 0 0 0 4px var(--cx-brass-inner); transition: box-shadow 0.15s; }
.cx-plaque:hover, .cx-plaque:focus-visible { box-shadow: inset 0 0 0 3px var(--cx-brass-inset), inset 0 0 0 4px var(--cx-brass-hover), 0 0 12px rgba(245, 165, 36, 0.25); outline: none; }
.cx-plaque.cx-can { cursor: pointer; border-color: var(--cx-brass-can); }
.cx-plaque.cx-have { border-color: var(--cx-brass-have-line); background: var(--cx-brass-have-bg);
  box-shadow: inset 0 0 0 3px var(--cx-brass-have-inset), inset 0 0 0 4px var(--cx-brass-can), 0 0 14px rgba(255, 210, 122, 0.3); color: var(--cx-brass-have-text); }
.cx-plaque.cx-unified { padding: 9px 14px 8px; border-width: 2px; }
.cx-rivet { position: absolute; top: 6px; width: 5px; height: 5px; border-radius: 50%; background: var(--cx-rivet); }
.cx-rivet-l { left: 6px; }
.cx-rivet-r { right: 6px; }
.cx-plaque-studs { display: flex; gap: 5px; }
.cx-stud { width: 9px; height: 9px; transform: rotate(45deg); border: 1px solid var(--cx-stud-line); }
.cx-plaque-name { font-size: 12.5px; font-weight: 700; letter-spacing: 1.2px; text-transform: uppercase; color: var(--cx-brass-name); }
.cx-plaque-text { font-size: 11px; line-height: 1.3; color: var(--cx-brass-sub); }
.cx-plaque.cx-have .cx-plaque-text { color: inherit; }
.cx-plaque-cost { font-size: 11px; color: var(--cx-brass-cost); letter-spacing: 1px; }
.cx-plaque-cost .cx-dim { color: var(--cx-brass-sub); }
@media (max-height: 860px) {
  .cx-side { padding-top: 16px; padding-bottom: 16px; }
  .cx-case-well { gap: 5px; padding: 6px; }
  .cx-plaque, .cx-plaque.cx-unified { padding: 4px 12px 3px; gap: 1px; }
  .cx-plaque-name { font-size: 11.5px; }
  .cx-plaque-text { font-size: 10px; line-height: 1.2; }
  .cx-plaque-cost { font-size: 10px; }
}
`;
