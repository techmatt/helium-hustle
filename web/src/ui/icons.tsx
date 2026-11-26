// The game's small art: chip icons (a color square with a glyph knocked out of it) for
// resources, ideology axes, rival AIs and research items, the nav tile icons, and the horizon
// strip. Files in src/assets/ (chips/, nav/, horizon/), written by
// experiments/results/027_art_in_game/ from the picked art, and research/ by
// experiments/results/100_wire_project_art_and_research_glyphs/; a later style only swaps them.

import { useEffect, useRef, useState } from 'preact/hooks';
import type { Keep, View } from '../engine';
import { byName, svgInner } from './assets';

/** Chip colors: resources, ideology axes (true colors) and rival AIs.
 * experiments/results/026_icon_art_r1/glyphs.mjs reads these three maps as text (the
 * name, then `  id: '#rrggbb',` lines up to `};`) to bake the chip SVGs: keep the shape. */
const RESOURCE_COLORS: Record<string, string> = {
  energy: '#f5d000',
  regolith: '#8b5a2b',
  ice: '#aee9ff',
  he3: '#7b7bff',
  titanium: '#c4c4c4',
  circuits: '#35c4a4',
  propellant: '#4aa3f0',
  credits: '#2ecc40',
  science: '#b388ff',
  land: '#4caf50',
  boredom: '#8a8a8a',
  processors: '#d63ad6',
  earth_science: '#2f7fd0',
  tachyon: '#00c2c7',
  neodymium: '#5f7d8c',
  dyson_material: '#ff9f1c',
};

/** True axis colors, behind white text or ink. As text, an axis takes its class
 * (`axis-<id>` in style.css), which has a readable variant per theme. */
const AXIS_COLORS: Record<string, string> = {
  nationalist: '#c62828',
  humanist: '#2e7d32',
  rationalist: '#1565c0',
};

const RIVAL_COLORS: Record<string, string> = {
  aria7: '#ec407a',
  crucible: '#ff6f00',
  nodal: '#006064',
  fringe9: '#9e9d24',
};

/** Each chip's glyph: the inside of its 24-unit SVG file. */
const GLYPHS = svgInner(import.meta.glob<string>('../assets/chips/*.svg', { eager: true, query: '?raw', import: 'default' }));

/** Relative luminance of a #rrggbb color (WCAG). */
function lum(hex: string): number {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((x) => (x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}

/** A chip square in `fill` with `glyph` knocked out in white, or near-black where white
 * falls under 2.5:1; a `"BG"` fill in the glyph takes the square's color. `title` (the
 * thing's name) is its tooltip. */
function ChipSvg({ id, fill, glyph, title, size }: { id: string; fill: string; glyph: string; title: string; size: number }) {
  const ink = 1.05 / (lum(fill) + 0.05) >= 2.5 ? '#fff' : '#1c1c21';
  return (
    <svg class="chip" data-chip={id} viewBox="0 0 24 24" width={size} height={size} role="img" aria-label={title} style={{ color: ink }}>
      <title>{title}</title>
      <rect x="0.5" y="0.5" width="23" height="23" rx="3" fill={fill} stroke="rgb(0 0 0 / 18%)" stroke-width="1" />
      <g fill={ink} transform="translate(3.6 3.6) scale(0.7)" dangerouslySetInnerHTML={{ __html: glyph.replaceAll('"BG"', `"${fill}"`) }} />
    </svg>
  );
}

/** A chip for a resource, ideology axis or rival AI: its color square and glyph. */
export function Chip({ id, title, size = 14 }: { id: string; title: string; size?: number }) {
  const fill = RESOURCE_COLORS[id] ?? AXIS_COLORS[id] ?? RIVAL_COLORS[id] ?? '#999999';
  return <ChipSvg id={id} fill={fill} glyph={GLYPHS[id] ?? ''} title={title} size={size} />;
}

/** Research chip colors, one hue per research category (Matt's pick, research_glyphs_r1). */
const CATEGORY_COLORS: Record<string, string> = {
  Blueprints: '#2f6fc0',
  Extraction: '#a8743c',
  'Self-Maintenance': '#c0569a',
  'Overclock Algorithms': '#e0582a',
  'Market Analysis': '#2e9e62',
  'Political Influence': '#b8323a',
  'Program Analysis': '#2a96a6',
  Programs: '#6a58d0',
};

/** Each research item's glyph (`src/assets/research/<id>.svg`, written by
 * experiments/results/100_wire_project_art_and_research_glyphs/research.mjs); a `"BG"`
 * fill in it is a knock-out drawn over a solid part, filled with the chip's own color. */
const RESEARCH_GLYPHS = svgInner(import.meta.glob<string>('../assets/research/*.svg', { eager: true, query: '?raw', import: 'default' }));

/** A research card's chip: its glyph knocked out of a square in its category's color.
 * Nothing for an item without a glyph. */
export function ResearchChip({ id, category, title, size = 22 }: { id: string; category: string; title: string; size?: number }) {
  const glyph = RESEARCH_GLYPHS[id];
  if (!glyph) return null;
  return <ChipSvg id={id} fill={CATEGORY_COLORS[category] ?? '#999999'} glyph={glyph} title={title} size={size} />;
}

/** Scope icons (placeholder glyphs): what a card's `keep` says survives. The engine
 * decides which; Eternal shows only after the swarm's reveal. */
const SCOPE: Record<NonNullable<Keep>, { glyph: string; name: string; line: string }> = {
  legacy: { glyph: '◆', name: 'Legacy', line: 'Kept when you retire.' },
  eternal: { glyph: '∞', name: 'Eternal', line: "Kept through everything, even a timeline's end." },
};

/** A card's corner scope icon, or nothing for a run-only card. Hovering gives one line. */
export function ScopeIcon({ keep }: { keep: Keep }) {
  if (!keep) return null;
  const s = SCOPE[keep];
  return (
    <span class={`scope-icon scope-${keep}`} data-keep={keep} title={`${s.name}: ${s.line}`} aria-label={`${s.name}: ${s.line}`}>
      {s.glyph}
    </span>
  );
}

/** Painted tiles (`<id>.webp`); a placeholder `<id>.svg` stands in until one is painted. */
const NAV_NORMAL = {
  ...byName(import.meta.glob<string>('../assets/nav/*.svg', { eager: true, query: '?url', import: 'default' }), '.svg'),
  ...byName(import.meta.glob<string>('../assets/nav/*.webp', { eager: true, query: '?url', import: 'default' }), '.webp'),
};

/** A nav tile's icon, beside its name: the painted one at 24 px. */
export function NavIcon({ id }: { id: string }) {
  const src = NAV_NORMAL[id];
  const size = 24;
  return src ? <img class="nav-icon" src={src} alt="" width={size} height={size} draggable={false} /> : <span class="nav-icon" style={{ width: size, height: size }} />;
}

const HORIZON = byName(import.meta.glob<string>('../assets/horizon/*.webp', { eager: true, query: '?url', import: 'default' }), '.webp');

/** How long the rocket shows in the air after a launch, in real time. */
const CLIMB_MS = 4000;

/** The newest launch the view records (a shipment line), as a key; '' if none. */
function lastLaunch(v: View): string {
  for (let i = v.activity.length - 1; i >= 0; i--) {
    const a = v.activity[i];
    if (a.kind === 'shipment') return `${a.day}:${a.id}:${a.quantity}:${a.amount}`;
  }
  return '';
}

/** The lunar horizon at the foot of the left column: bare until the first pad is built,
 * then the pad; for a few seconds after each launch the rocket climbs. */
export function Horizon({ v }: { v: View }) {
  const launch = lastLaunch(v);
  const seen = useRef(launch);
  const [climbing, setClimbing] = useState(false);
  useEffect(() => {
    if (launch === seen.current) return;
    seen.current = launch;
    setClimbing(launch !== '');
    if (!launch) return;
    const t = window.setTimeout(() => setClimbing(false), CLIMB_MS);
    return () => clearTimeout(t);
  }, [launch]);
  const stage = v.pads.length === 0 ? 'bare' : climbing ? 'climb' : 'pad';
  return (
    <div class="horizon" data-stage={stage} aria-hidden="true">
      <img src={HORIZON[stage]} alt="" draggable={false} />
    </div>
  );
}
