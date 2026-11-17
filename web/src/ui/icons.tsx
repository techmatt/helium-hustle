// The game's small art: chip icons (a color square with a glyph knocked out of it) for
// resources, ideology axes and rival AIs, the nav tile icons, and the horizon strip.
// Files in src/assets/ (chips/, nav/, horizon/), written by
// experiments/results/027_art_in_game/ from the picked art; a later style only swaps them.

import { useEffect, useRef, useState } from 'preact/hooks';
import type { View } from '../engine';

/** Chip colors: resources, ideology axes (true colors) and rival AIs. */
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
};

/** True axis colors, behind white text or ink. As text, an axis takes its class
 * (`axis-<id>` in style.css), which has a readable variant per theme. */
export const AXIS_COLORS: Record<string, string> = {
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

const byName = <T,>(files: Record<string, T>, ext: string): Record<string, T> =>
  Object.fromEntries(Object.entries(files).map(([f, x]) => [f.slice(f.lastIndexOf('/') + 1, -ext.length), x]));

/** Each chip's glyph: the inside of its 24-unit SVG file. */
const GLYPHS = Object.fromEntries(
  Object.entries(byName(import.meta.glob<string>('../assets/chips/*.svg', { eager: true, query: '?raw', import: 'default' }), '.svg')).map(
    ([id, svg]) => [id, svg.slice(svg.indexOf('>') + 1, svg.lastIndexOf('</svg>'))],
  ),
);

/** Relative luminance of a #rrggbb color (WCAG). */
function lum(hex: string): number {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((x) => (x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}

/** A chip: the id's color square with its glyph knocked out in white, or near-black where
 * white falls under 2.5:1. `title` (the thing's name) is its tooltip. */
export function Chip({ id, title, size = 14 }: { id: string; title: string; size?: number }) {
  const fill = RESOURCE_COLORS[id] ?? AXIS_COLORS[id] ?? RIVAL_COLORS[id] ?? '#999999';
  const ink = 1.05 / (lum(fill) + 0.05) >= 2.5 ? '#fff' : '#1c1c21';
  return (
    <svg class="chip" data-chip={id} viewBox="0 0 24 24" width={size} height={size} role="img" aria-label={title} style={{ color: ink }}>
      <title>{title}</title>
      <rect x="0.5" y="0.5" width="23" height="23" rx="3" fill={fill} stroke="rgb(0 0 0 / 18%)" stroke-width="1" />
      <g fill={ink} transform="translate(3.6 3.6) scale(0.7)" dangerouslySetInnerHTML={{ __html: GLYPHS[id] ?? '' }} />
    </svg>
  );
}

const NAV_NORMAL = byName(import.meta.glob<string>('../assets/nav/*.webp', { eager: true, query: '?url', import: 'default' }), '.webp');
const NAV_COMPACT = byName(import.meta.glob<string>('../assets/nav/*-compact.svg', { eager: true, query: '?url', import: 'default' }), '-compact.svg');

/** A nav tile's icon: the painted one at normal size, the compact chip at 18 px. */
export function NavIcon({ id, compact }: { id: string; compact: boolean }) {
  const src = (compact ? NAV_COMPACT : NAV_NORMAL)[id];
  const size = compact ? 18 : 34;
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
