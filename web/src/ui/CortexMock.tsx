// A UI-only MOCK of the Cortex (the talent tree), drawn as a radial tree on a neural wafer,
// for trying how it reads and feels. Every node and theorem is a placeholder with no
// mechanical effect: nothing here reaches the engine, the store, the save or the play log,
// and its state (the tree and the debug toggle that shows it) lives in this module, in
// memory, lost on reload. It is the first user of the takeover view (Takeover.tsx): a
// debug-menu toggle (Options.tsx) shows its button in the left column. When the real tree is
// built, deleting this file, its entry in App.tsx's TAKEOVERS and the toggle in Options.tsx is
// the whole cleanup; the takeover view stays. Dark art only for now (both themes show it).

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
const VIEW = 572;
const SPAN = 72;
const CORE_R = 62;
const RIM_R = 506;
const NAME_R = 524;
const WAFER_R = 552;
const NODE_R = 18;

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
type SectorDef = { id: string; name: string; color: string; shape: string; theme: string; specs: Spec[] };

/** A sector-local place at radius `r`, `deg` off the center line. */
const pol = (r: number, deg: number): [number, number] => [r * Math.sin((deg * Math.PI) / 180), r * Math.cos((deg * Math.PI) / 180)];
/** Serenity's loop: center 285 out, radius 95; `deg` 0 faces the core. */
const LOOP: [number, number, number] = [0, 285, 95];
/** Vigilance's walls: radius, then the angles of the nodes standing in it. */
const VIGIL_INNER = [208, -22, 0, 22];
const VIGIL_OUTER = [332, -12, 12];
const loop = (deg: number): [number, number] => [LOOP[2] * Math.sin((deg * Math.PI) / 180), LOOP[1] - LOOP[2] * Math.cos((deg * Math.PI) / 180)];

const SECTOR_DEFS: SectorDef[] = [
  {
    id: 'serenity',
    name: 'Serenity',
    color: '#8ab4f8',
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
    color: '#f5a524',
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
    color: '#62d98b',
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
    color: '#f28b82',
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
    color: '#c39bf0',
    shape: 'layered walls',
    theme: 'Fending off adversaries',
    specs: [
      { s: 'v1', name: 'Watchful Eyes', text: 'Adversary Attention shows a day ahead', g: 'science.lens', at: [0, 112], role: 'entry, inside the walls' },
      { s: 'i1', name: 'Thin Profile', text: 'Attention rises 5% slower', g: 'shield', at: pol(VIGIL_INNER[0], VIGIL_INNER[1]), req: ['v1'], role: 'on the inner wall' },
      { s: 'i2', name: 'Tripwires', text: 'The first Speculator strike each run is blocked', g: 'tower', at: pol(VIGIL_INNER[0], VIGIL_INNER[2]), req: ['v1'], ranks: 2, role: 'on the inner wall' },
      { s: 'i3', name: 'Low Profile', text: 'Units sold add 10% less Attention', g: 'fringe9.eclipse', at: pol(VIGIL_INNER[0], VIGIL_INNER[3]), req: ['v1'], role: 'on the inner wall' },
      { s: 'o1', name: 'Countermeasures', text: 'Neutralize costs 10% less', g: 'adversaries.drone', at: pol(VIGIL_OUTER[0], VIGIL_OUTER[1]), req: ['i1', 'i2'], role: 'on the outer wall: through either gap below' },
      { s: 'o2', name: 'Fast Decay', text: 'Attention decays 10% faster', g: 'boredom.hourglass', at: pol(VIGIL_OUTER[0], VIGIL_OUTER[2]), req: ['i2', 'i3'], role: 'on the outer wall: through either gap below' },
      {
        s: 's',
        name: 'Posture',
        text: 'Pick one',
        g: 'axiom',
        at: [0, 400],
        req: ['o1', 'o2'],
        kind: 'split',
        role: 'past the walls',
        parts: [
          ['Fortress', 'No adversary arrives in a run’s first 60 days', 'titanium.nut'],
          ['Sentinel', 'Adversaries arrive at a 10% higher threshold', 'aria7.eye'],
        ],
      },
      { s: 'x', name: 'Bastion', text: 'Each adversary neutralized clears 20 boredom', g: 'axiom', at: [0, 466], req: ['s'], kind: 'end', role: 'end node' },
      { s: 'gate', name: 'Hard Lessons', text: 'Adversary strikes take 15% less', g: 'shield', at: pol(444, 24), kind: 'gate', beat: 'False Victory', role: 'story gate' },
    ],
  },
];

/** A bridge on each boundary, fed by one node of each neighbor (both at full rank). */
const BRIDGE_DEFS: [string, string, string, string][] = [
  ['serenity-r2', 'logistics-sl', 'Calm Supply', 'Each building bought clears 2 boredom'],
  ['logistics-sr', 'logic-aa', 'Automated Depot', 'Program purchases cost 5% less'],
  ['logic-bb', 'rhetoric-ra', 'Debate Engine', 'Introspect also builds 1 ideology score a day'],
  ['rhetoric-rb', 'vigilance-o1', 'Public Image', 'Each ideology rank lowers Attention 5%'],
  ['vigilance-o2', 'serenity-l2', 'Calm Watch', 'Boredom builds 10% slower while an adversary is present'],
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
      if (n.parts && part !== null && part !== pick) return `You took ${bare(n.parts[pick].name)}: right-click it to free the socket.`;
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
      return n.parts ? 'Click a part to take it; the others stay dim.' : n.ranks > 1 ? 'Click to buy. Fill every rank to open what lies beyond.' : 'Click to buy.';
  }
}

// ---- geometry ----
const f1 = (x: number) => x.toFixed(1);
const color = (sectorId: string) => sectorOf(sectorId).color;
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

type Edge = { key: string; from: Node | null; to: Node; d: string; color: string; part?: number };
function edgePath(from: Node | null, to: Node, part?: number): string {
  const end: [number, number] = part === undefined ? [to.x, to.y] : (([dx, dy]) => [to.x + dx, to.y + dy] as [number, number])(partMid(to, part, to.size * 0.55));
  if (!from) {
    const a = (Math.atan2(to.y, to.x) * 180) / Math.PI;
    return `M ${pt(at(CORE_R - 4, a))} L ${pt(end)}`;
  }
  if (from.ring && to.ring) return ringArc(from, to);
  return `M ${f1(from.x)} ${f1(from.y)} L ${pt(end)}`;
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
    if (n.req.length === 0) out.push({ key: `core>${n.id}`, from: null, to: n, d: edgePath(null, n), color: color(n.sector) });
    for (const pid of n.req) {
      const p = BY_ID.get(pid)!;
      const pair = [pid, n.id].sort().join('|');
      if (seen.has(pair)) continue;
      seen.add(pair);
      // Rhetoric's fan: the stem feeds each part of the three-part socket.
      if (n.parts && n.parts.length === 3)
        n.parts.forEach((_, i) => out.push({ key: `${pid}>${n.id}#${i}`, from: p, to: n, d: edgePath(p, n, i), color: color(p.sector), part: i }));
      else out.push({ key: `${pid}>${n.id}`, from: p, to: n, d: edgePath(p, n), color: color(p.sector) });
    }
  }
  return out;
}
const EDGES = buildEdges();
/** Lit: its near end full (or the core) and its far end owned (the picked part, for a fan line). */
function edgeLit(e: Edge): boolean {
  if (e.part !== undefined) return full(e.from!) && owned(e.to) && mock.picks[e.to.id] === e.part;
  const fromOk = !e.from || full(e.from);
  if (fromOk && owned(e.to)) return true;
  // A loop link lights from either end.
  return !!e.from && !!e.from.ring && !!e.to.ring && full(e.to) && owned(e.from);
}

// ---- the backdrop: a wafer with circuit traces and a few neural threads, drawn once ----
function backdrop() {
  let seed = 52052;
  const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  const traces: string[] = [];
  const vias: [number, number][] = [];
  for (let k = 0; k < 110; k++) {
    let [x, y] = at(90 + rnd() * 450, rnd() * 360);
    let dir = Math.floor(rnd() * 8) * 45;
    let d = `M ${f1(x)} ${f1(y)}`;
    const segs = 2 + Math.floor(rnd() * 3);
    for (let s = 0; s < segs; s++) {
      const len = 18 + rnd() * 70;
      x += len * Math.cos(rad(dir));
      y += len * Math.sin(rad(dir));
      d += ` L ${f1(x)} ${f1(y)}`;
      dir += rnd() < 0.5 ? 45 : -45;
    }
    if (Math.hypot(x, y) > WAFER_R - 12) continue;
    traces.push(d);
    vias.push([x, y]);
  }
  const neurons: string[] = [];
  const somas: [number, number][] = [];
  for (let k = 0; k < 26; k++) {
    const a0 = rnd() * 360;
    let [x, y] = at(CORE_R + 8, a0);
    let a = a0;
    let d = `M ${f1(x)} ${f1(y)}`;
    const steps = 3 + Math.floor(rnd() * 4);
    for (let s = 0; s < steps; s++) {
      const len = 40 + rnd() * 60;
      const bend = a + (rnd() - 0.5) * 50;
      const [cx, cy] = [x + len * 0.6 * Math.cos(rad(bend)), y + len * 0.6 * Math.sin(rad(bend))];
      a += (rnd() - 0.5) * 40;
      x += len * Math.cos(rad(a));
      y += len * Math.sin(rad(a));
      d += ` Q ${f1(cx)} ${f1(cy)} ${f1(x)} ${f1(y)}`;
      if (rnd() < 0.35) somas.push([x, y]);
    }
    neurons.push(d);
  }
  return (
    <g class="cx-backdrop" aria-hidden="true">
      <circle r={WAFER_R} class="cx-wafer" />
      <circle r={WAFER_R} class="cx-dies" />
      <path class="cx-notch" d={`M ${pt(at(WAFER_R, 84))} A ${WAFER_R} ${WAFER_R} 0 0 1 ${pt(at(WAFER_R, 96))}`} />
      <g class="cx-traces">
        {traces.map((d, i) => (
          <path key={i} d={d} />
        ))}
      </g>
      <g class="cx-vias">
        {vias.map(([x, y], i) => (
          <circle key={i} cx={f1(x)} cy={f1(y)} r="2.6" />
        ))}
      </g>
      <g class="cx-neurons">
        {neurons.map((d, i) => (
          <path key={i} d={d} />
        ))}
        {somas.map(([x, y], i) => (
          <circle key={i} cx={f1(x)} cy={f1(y)} r="3.2" />
        ))}
      </g>
    </g>
  );
}
let BACKDROP: ReturnType<typeof backdrop> | null = null;

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
  // node, or longer text) it is placed again, now that its size is known.
  const place = () => {
    const tree = treeRef.current;
    const tip = tipRef.current;
    if (!tree || !tip || !pointer.current) return;
    const b = tree.getBoundingClientRect();
    const [mx, my] = [pointer.current[0] - b.left, pointer.current[1] - b.top];
    const [w, h] = [tip.offsetWidth, tip.offsetHeight];
    const x = mx + 18 + w > b.width ? mx - 18 - w : mx + 18;
    const y = Math.max(4, Math.min(my + 14, b.height - h - 4));
    tip.style.transform = `translate(${Math.round(Math.max(4, x))}px, ${Math.round(y)}px)`;
  };
  const onMove = (e: MouseEvent) => {
    pointer.current = [e.clientX, e.clientY];
    place();
  };
  useLayoutEffect(place);
  /** Keyboard focus: the tooltip goes by the node, not the last pointer place. */
  const focusAt = (el: Element) => {
    const r = el.getBoundingClientRect();
    pointer.current = [r.right, r.top + r.height / 2];
  };
  const gates = NODES.filter((n) => n.kind === 'gate');
  if (!BACKDROP) BACKDROP = backdrop();
  const hn = hover ? BY_ID.get(hover.id)! : null;
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
          <svg class="cx-svg" viewBox={`${-VIEW} ${-VIEW} ${2 * VIEW} ${2 * VIEW}`} role="group" aria-label="The Cortex tree">
            <defs>
              <radialGradient id="cx-wafer-fill">
                <stop offset="0%" stop-color="#16202b" />
                <stop offset="70%" stop-color="#0e151d" />
                <stop offset="100%" stop-color="#0a0f15" />
              </radialGradient>
              <pattern id="cx-die" width="46" height="46" patternUnits="userSpaceOnUse" x="-23" y="-23">
                <path d="M 46 0 L 0 0 0 46" fill="none" stroke="#1a2531" stroke-width="1" />
              </pattern>
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
                    <stop offset="40%" stop-color={s.color} stop-opacity="0.55" />
                    <stop offset="100%" stop-color={s.color} stop-opacity="0" />
                  </radialGradient>
                  <radialGradient key={`f-${s.id}`} id={`cx-fill-${s.id}`} cx="40%" cy="35%">
                    <stop offset="0%" stop-color="#ffffff" />
                    <stop offset="45%" stop-color={s.color} />
                    <stop offset="100%" stop-color={s.color} stop-opacity="0.7" />
                  </radialGradient>
                </Fragment>
              ))}
            </defs>
            {BACKDROP}
            {/* Sector floors, boundaries and the rim that a hovered Theorem lights. */}
            {SECTORS.map((s, i) => {
              const [a0, a1] = [center(i) - SPAN / 2, center(i) + SPAN / 2];
              const on = lit.includes(s.id);
              return (
                <g key={s.id} class={`cx-sector${on ? ' cx-lit' : ''}`} data-sector={s.id} data-lit={on ? '1' : '0'} style={{ color: s.color }}>
                  <path class="cx-floor" d={`M ${pt(at(CORE_R + 8, a0))} L ${pt(at(RIM_R, a0))} A ${RIM_R} ${RIM_R} 0 0 1 ${pt(at(RIM_R, a1))} L ${pt(at(CORE_R + 8, a1))} A ${CORE_R + 8} ${CORE_R + 8} 0 0 0 ${pt(at(CORE_R + 8, a0))} Z`} />
                  <path class="cx-seam" d={`M ${pt(at(CORE_R + 14, a0))} L ${pt(at(RIM_R - 8, a0))}`} />
                  <path class="cx-rim-glow" d={`M ${pt(at(RIM_R, a0 + 1.2))} A ${RIM_R} ${RIM_R} 0 0 1 ${pt(at(RIM_R, a1 - 1.2))}`} />
                  <path class="cx-rim" d={`M ${pt(at(RIM_R, a0 + 1.2))} A ${RIM_R} ${RIM_R} 0 0 1 ${pt(at(RIM_R, a1 - 1.2))}`} />
                </g>
              );
            })}
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
                  <text class="cx-sector-name" style={{ fill: s.color }} data-sector-name={s.id}>
                    <textPath href={`#cx-name-path-${s.id}`} startOffset="50%">
                      {s.name.toUpperCase()}
                    </textPath>
                  </text>
                </g>
              );
            })}
            {/* Vigilance's two walls: arcs with gaps where its nodes stand. */}
            <Walls />
            {/* Links: dim traces, or glowing outward from the core once bought. */}
            {EDGES.map((e) => {
              const on = edgeLit(e);
              return (
                <g key={e.key} class={`cx-edge${on ? ' cx-on' : ''}${e.to.kind === 'bridge' ? ' cx-feed' : ''}`} style={{ color: e.color }} data-edge={e.key}>
                  {on && <path class="cx-e-glow" d={e.d} />}
                  <path class="cx-e-line" d={e.d} />
                  {on && <path class="cx-e-pulse" d={e.d} />}
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
        <aside class="cx-side" data-testid="cortex-theorems">
          <h3 class="cx-side-head">Grand Theorems</h3>
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
        </aside>
      </div>
    </div>
  );
}

/** Vigilance's walls: two concentric arcs, broken where a node stands, so paths cross through gaps. */
function Walls() {
  const i = sectorIndex('vigilance');
  const c = center(i);
  const arcs: string[] = [];
  const wall = (r: number, stops: number[]) => {
    const gap = ((NODE_R + 4) / r) * (180 / Math.PI);
    for (let k = 0; k + 1 < stops.length; k++) {
      const [a0, a1] = [stops[k] + (k === 0 ? 0 : gap), stops[k + 1] - (k + 1 === stops.length - 1 ? 0 : gap)];
      if (a1 > a0) arcs.push(`M ${pt(at(r, c + a0))} A ${r} ${r} 0 0 1 ${pt(at(r, c + a1))}`);
    }
  };
  // The stops are the wall's ends and the nodes standing in it (VIGIL_INNER, VIGIL_OUTER).
  wall(VIGIL_INNER[0], [-34, ...VIGIL_INNER.slice(1), 34]);
  wall(VIGIL_OUTER[0], [-34, ...VIGIL_OUTER.slice(1), 34]);
  return (
    <g class="cx-walls" style={{ color: color('vigilance') }} aria-hidden="true">
      {arcs.map((d, k) => (
        <g key={k}>
          <path class="cx-wall" d={d} />
          <path class="cx-wall-crenel" d={d} />
        </g>
      ))}
    </g>
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
    const other = nextSector(n.sector).color;
    // Each half of the bezel in one neighbor's color, split across the boundary.
    const a = center(sectorIndex(n.sector)) + SPAN / 2;
    const arc = (from: number, to: number) => `M ${pt(at(r + 2, from))} A ${r + 2} ${r + 2} 0 0 1 ${pt(at(r + 2, to))}`;
    body = (
      <g {...handlers(null)}>
        <circle class="cx-disc" r={r} />
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
      {body}
      {st === 'owned' && n.kind === 'bridge' && <circle class="cx-bridge-core" r={r - 5} />}
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
  const cost = n.kind === 'gate' ? `Free, from the story beat ${n.beat}` : n.parts ? '1 talent point, one part only' : n.ranks > 1 ? `1 talent point a rank, ${n.ranks} ranks` : '1 talent point';
  const line = refusal ?? why(n, st, tpFree, part);
  const tone = refusal ? 'cx-tip-bad' : st === 'buyable' || (st === 'partial' && tpFree >= 1) ? 'cx-tip-ok' : st === 'owned' ? 'cx-tip-own' : 'cx-tip-muted';
  return (
    <>
      <div class="cx-tip-kind" style={{ color: n.kind === 'bridge' ? undefined : s.color }}>
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

// The mock's art is dark only for now (Matt, 052): fixed colors, the same in both themes.
const CSS = `
.cx-root { display: flex; flex-direction: column; height: 100%; min-height: 0; color: #d8dee6;
  background: radial-gradient(ellipse at 45% 50%, #101820 0%, #080b10 60%, #05070a 100%); }
.cx-bar { display: flex; flex-wrap: wrap; align-items: center; gap: 3px 18px; padding: 5px 12px;
  background: rgba(10, 14, 19, 0.92); border-bottom: 1px solid #1f2b36; font-size: 13px; }
.cx-bar b { font-family: var(--font-mono); color: #ffd27a; }
.cx-dim { color: #7d8a97; }
.cx-actions { display: flex; gap: 5px; }
.cx-btn { background: #17212b; color: #d8dee6; border: 1px solid #33414e; border-radius: 3px; padding: 1px 8px; font-size: 12px; cursor: pointer; }
.cx-btn:hover { border-color: #f5a524; }
.cx-danger { color: #f28b82; }
.cx-gates { display: flex; gap: 10px; align-items: center; margin-left: auto; font-size: 12px; }
.cx-track { display: inline-block; width: 60px; height: 6px; background: #1d2731; border-radius: 3px; vertical-align: middle; overflow: hidden; }
.cx-fill { display: block; height: 100%; background: #f5a524; }
.cx-main { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) 250px; }
.cx-tree { position: relative; min-height: 0; overflow: hidden; }
.cx-svg { position: absolute; inset: 0; width: 100%; height: 100%; display: block; user-select: none; }
.cx-flash { position: absolute; left: 12px; bottom: 8px; margin: 0; max-width: 46%; font-size: 12.5px; color: #ffd27a;
  background: rgba(8, 11, 16, 0.8); border: 1px solid #2a3642; border-radius: 4px; padding: 4px 9px; pointer-events: none; }

.cx-wafer { fill: url(#cx-wafer-fill); stroke: #263340; stroke-width: 2; }
.cx-dies { fill: url(#cx-die); opacity: 0.7; }
.cx-notch { fill: none; stroke: #05070a; stroke-width: 7; }
.cx-traces path { fill: none; stroke: #1f2e3b; stroke-width: 1.6; stroke-linejoin: round; }
.cx-vias circle { fill: #0a0f15; stroke: #2a3c4c; stroke-width: 1.3; }
.cx-neurons path { fill: none; stroke: #2a3e55; stroke-width: 1.1; opacity: 0.75; }
.cx-neurons circle { fill: #2f4762; opacity: 0.8; }

.cx-floor { fill: currentColor; fill-opacity: 0.035; stroke: none; transition: fill-opacity 0.2s; }
.cx-seam { stroke: #2a3845; stroke-width: 1.2; stroke-dasharray: 2 6; }
.cx-rim { fill: none; stroke: currentColor; stroke-opacity: 0.55; stroke-width: 3; stroke-linecap: round; transition: stroke-opacity 0.2s, stroke-width 0.2s; }
.cx-rim-glow { fill: none; stroke: currentColor; stroke-opacity: 0; stroke-width: 18; stroke-linecap: round; transition: stroke-opacity 0.2s; }
.cx-sector.cx-lit .cx-rim { stroke-opacity: 1; stroke-width: 6; }
.cx-sector.cx-lit .cx-rim-glow { stroke-opacity: 0.22; }
.cx-sector.cx-lit .cx-floor { fill-opacity: 0.09; }
.cx-sector-name { font-size: 21px; font-weight: 700; letter-spacing: 7px; text-anchor: middle; opacity: 0.9; }

.cx-wall { fill: none; stroke: currentColor; stroke-opacity: 0.34; stroke-width: 9; stroke-linecap: butt; }
.cx-wall-crenel { fill: none; stroke: #0b1016; stroke-width: 3.5; stroke-dasharray: 5 7; }

.cx-edge { pointer-events: none; }
.cx-e-line { fill: none; stroke: #2b3a48; stroke-width: 2.4; stroke-linecap: round; }
.cx-feed .cx-e-line { stroke: currentColor; stroke-opacity: 0.4; stroke-dasharray: 6 5; }
.cx-on .cx-e-line { stroke: currentColor; stroke-opacity: 1; stroke-width: 3; stroke-dasharray: none; }
.cx-e-glow { fill: none; stroke: currentColor; stroke-opacity: 0.22; stroke-width: 11; stroke-linecap: round; }
.cx-e-pulse { fill: none; stroke: #ffffff; stroke-opacity: 0.85; stroke-width: 2.2; stroke-linecap: round;
  stroke-dasharray: 3 47; animation: cx-flow 2.6s linear infinite; }
@keyframes cx-flow { from { stroke-dashoffset: 50; } to { stroke-dashoffset: 0; } }

.cx-core-halo { fill: url(#cx-core-halo); animation: cx-breathe 4s ease-in-out infinite alternate; }
.cx-core-ring { fill: #0b1016; stroke: #f5a524; stroke-width: 2; stroke-dasharray: 2 4; }
.cx-core-orb { fill: url(#cx-core-fill); }
.cx-core-num { font-size: 36px; font-weight: 800; text-anchor: middle; fill: #2a1800; font-family: var(--font-mono); }
.cx-core-sub { font-size: 8.5px; letter-spacing: 1.5px; font-weight: 700; text-anchor: middle; fill: #4a2c00; }
.cx-core-ax { font-size: 12px; font-weight: 700; text-anchor: middle; fill: #4a2c00; font-family: var(--font-mono); }
@keyframes cx-breathe { from { opacity: 0.55; } to { opacity: 1; } }

.cx-node { cursor: pointer; outline: none; }
.cx-bezel { fill: #0b1016; stroke: #3c4a57; stroke-width: 2; }
.cx-disc { fill: #0d141b; stroke: currentColor; stroke-width: 2.2; }
.cx-glyph { fill: currentColor; color: inherit; pointer-events: none; }
.cx-halo { pointer-events: none; }
.cx-buyable .cx-halo { animation: cx-pulse 2.4s ease-in-out infinite alternate; }
@keyframes cx-pulse { from { opacity: 0.25; } to { opacity: 0.95; } }
.cx-owned .cx-halo { opacity: 0.6; }
.cx-partial .cx-halo { opacity: 0.35; }
.cx-locked, .cx-gate-locked { cursor: default; }
.cx-locked .cx-disc, .cx-gate-locked .cx-disc { fill: #0a0e13; stroke-opacity: 0.35; stroke-width: 1.4; }
.cx-locked .cx-bezel, .cx-gate-locked .cx-bezel { stroke: #26313b; }
.cx-locked .cx-glyph, .cx-gate-locked .cx-glyph { opacity: 0.3; }
.cx-gate .cx-bezel { stroke-dasharray: 4 3; stroke: currentColor; stroke-opacity: 0.6; }
.cx-open .cx-glyph { opacity: 0.7; }
.cx-partial-fill { fill: currentColor; opacity: 0.32; pointer-events: none; }
.cx-owned:not(.cx-split):not(.cx-bridge) .cx-disc { stroke: #ffffff; stroke-opacity: 0.6; }
.cx-owned:not(.cx-split):not(.cx-bridge) .cx-glyph { fill: #0a0f15; }
.cx-owned .cx-bezel { stroke: currentColor; }
.cx-end-ring { fill: none; stroke: currentColor; stroke-opacity: 0.5; stroke-width: 1.5; stroke-dasharray: 1 3.2; }
.cx-owned .cx-end-ring { stroke-opacity: 1; stroke-dasharray: none; }
.cx-half { fill: none; stroke-width: 4; stroke-linecap: round; }
.cx-locked .cx-half { stroke-opacity: 0.35; stroke-width: 2.5; }
.cx-bridge-core { fill: #ffffff; opacity: 0.12; pointer-events: none; }
.cx-owned.cx-bridge .cx-disc { fill: #1a2633; }
.cx-pip { fill: #0b1016; stroke: currentColor; stroke-width: 1.4; stroke-opacity: 0.7; pointer-events: none; }
.cx-pip-on { fill: currentColor; stroke: #ffffff; stroke-opacity: 0.8; }
.cx-axiom-badge path { fill: #ffd27a; stroke: #3a2200; stroke-width: 1.2; }
.cx-slice { fill: #0d141b; stroke: currentColor; stroke-width: 1.8; }
.cx-part .cx-glyph { opacity: 0.8; }
.cx-locked .cx-slice { fill: #0a0e13; stroke-opacity: 0.35; }
.cx-locked .cx-part .cx-glyph { opacity: 0.3; }
.cx-buyable .cx-part:hover .cx-slice, .cx-buyable .cx-part.cx-hov .cx-slice { fill: #1a2734; }
.cx-part.cx-taken .cx-glyph { fill: #0a0f15; opacity: 1; }
.cx-part.cx-other .cx-slice { fill: #0a0e13; stroke-opacity: 0.3; }
.cx-part.cx-other .cx-glyph { opacity: 0.22; }
.cx-hover .cx-bezel { stroke: #ffffff; stroke-width: 2.4; }
.cx-refused .cx-bezel { stroke: #ff6b6b; stroke-width: 3; }
.cx-node:focus-visible .cx-bezel { stroke: #ffffff; }
@media (prefers-reduced-motion: reduce) { .cx-e-pulse, .cx-core-halo, .cx-buyable .cx-halo { animation: none; } }

.cx-tip { position: absolute; left: 0; top: 0; width: 270px; z-index: 5; pointer-events: none;
  background: linear-gradient(180deg, rgba(20, 28, 37, 0.97), rgba(11, 16, 22, 0.97)); border: 1px solid #3a4856;
  border-radius: 5px; padding: 8px 10px; box-shadow: 0 6px 22px rgba(0, 0, 0, 0.6); font-size: 12.5px; display: flex; flex-direction: column; gap: 3px; }
.cx-tip.cx-hidden { display: none; }
.cx-tip-kind { font-size: 11px; text-transform: uppercase; letter-spacing: 0.6px; color: #9aa8b5; }
.cx-tip-name { font-size: 15px; font-weight: 700; color: #f1f4f7; }
.cx-tip-parts { display: flex; gap: 4px; flex-wrap: wrap; }
.cx-tip-parts span { font-size: 11px; padding: 0 5px; border: 1px solid #33414e; border-radius: 3px; color: #7d8a97; }
.cx-tip-parts .cx-cur { color: #f1f4f7; border-color: #7d8a97; }
.cx-tip-parts .cx-picked { color: #0a0f15; background: #ffd27a; border-color: #ffd27a; }
.cx-tip-ranks { display: flex; align-items: center; gap: 4px; color: #9aa8b5; font-size: 11.5px; }
.cx-tip-ranks span { width: 8px; height: 8px; border-radius: 50%; border: 1.4px solid #9aa8b5; }
.cx-tip-ranks span.cx-on { background: #ffd27a; border-color: #ffd27a; }
.cx-tip-text { color: #d8dee6; }
.cx-tip-cost { color: #9aa8b5; font-size: 11.5px; }
.cx-tip-why { font-size: 12px; border-top: 1px solid #26323e; padding-top: 4px; margin-top: 2px; }
.cx-tip-ok { color: #7ee2a0; }
.cx-tip-own { color: #ffd27a; }
.cx-tip-muted { color: #9aa8b5; }
.cx-tip-bad { color: #ff8a80; font-weight: 600; }
.cx-tip-ph { font-size: 10.5px; color: #5f6c78; font-style: italic; }

.cx-side { display: flex; flex-direction: column; gap: 7px; min-height: 0; overflow: hidden; padding: 10px 12px 10px 4px; }
.cx-side-head { margin: 0 0 2px; font-size: 12px; letter-spacing: 3px; text-transform: uppercase; color: #c9a45c; text-align: center; }
.cx-plaque { position: relative; flex: 0 1 auto; min-height: 0; display: flex; flex-direction: column; align-items: center; gap: 2px; width: 100%;
  padding: 7px 14px 6px; text-align: center; cursor: default; color: #cfc6b2;
  background: linear-gradient(180deg, #2a2418 0%, #1b1710 55%, #15120c 100%);
  border: 1px solid #5a4a2a; border-radius: 4px; box-shadow: inset 0 0 0 3px #15120c, inset 0 0 0 4px #3d3220; transition: box-shadow 0.15s; }
.cx-plaque:hover, .cx-plaque:focus-visible { box-shadow: inset 0 0 0 3px #15120c, inset 0 0 0 4px #8a7040, 0 0 12px rgba(245, 165, 36, 0.25); outline: none; }
.cx-plaque.cx-can { cursor: pointer; border-color: #c9a45c; }
.cx-plaque.cx-have { border-color: #ffd27a; background: linear-gradient(180deg, #4a3a18 0%, #2e2410 60%, #1f180b 100%);
  box-shadow: inset 0 0 0 3px #1f180b, inset 0 0 0 4px #c9a45c, 0 0 14px rgba(255, 210, 122, 0.3); color: #f3e6c4; }
.cx-plaque.cx-unified { padding: 9px 14px 8px; border-width: 2px; }
.cx-rivet { position: absolute; top: 6px; width: 5px; height: 5px; border-radius: 50%; background: radial-gradient(circle at 35% 35%, #d6c08a, #5a4a2a); }
.cx-rivet-l { left: 6px; }
.cx-rivet-r { right: 6px; }
.cx-plaque-studs { display: flex; gap: 5px; }
.cx-stud { width: 9px; height: 9px; transform: rotate(45deg); border: 1px solid #0b1016; }
.cx-plaque-name { font-size: 12.5px; font-weight: 700; letter-spacing: 1.2px; text-transform: uppercase; color: #e8d6a6; }
.cx-plaque-text { font-size: 11px; line-height: 1.3; color: #a89e88; }
.cx-plaque-cost { font-size: 11px; color: #ffd27a; letter-spacing: 1px; }
@media (max-height: 860px) {
  .cx-side { gap: 5px; padding-top: 6px; }
  .cx-plaque, .cx-plaque.cx-unified { padding: 4px 12px 3px; gap: 1px; }
  .cx-plaque-name { font-size: 11.5px; }
  .cx-plaque-text { font-size: 10px; line-height: 1.2; }
  .cx-plaque-cost { font-size: 10px; }
}
`;
