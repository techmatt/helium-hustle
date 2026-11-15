// The Earthrise art, drawn inline: the Earth in the sky over the center column and the
// lunar horizon with the habitat at the foot of the left column. Its fills come from the
// theme tokens in style.css (the Earth keeps its own true colors in both themes).
//
// The sky is the home for a later live view of the base (Flash/3D): replace what <Sky>
// draws and keep its box (style.css `.sky`), so nothing else on the page moves.

const STARS: [number, number, number][] = [
  [4, 18, 1], [11, 62, 0.8], [17, 30, 1.2], [26, 80, 0.8], [33, 12, 0.9], [41, 50, 1.1],
  [48, 88, 0.7], [55, 24, 1], [61, 66, 0.8], [68, 40, 1.2], [74, 10, 0.8], [79, 92, 0.9],
  [86, 36, 0.8], [93, 70, 1], [97, 20, 0.7],
];

export function Sky() {
  return (
    <div class="sky" aria-hidden="true">
      <svg class="stars" width="100%" height="100%">
        {STARS.map(([x, y, r], i) => (
          <circle key={i} cx={`${x}%`} cy={`${y}%`} r={r} />
        ))}
      </svg>
      <Earth />
    </div>
  );
}

function Earth() {
  return (
    <svg class="earth" viewBox="0 0 200 200">
      <defs>
        <radialGradient id="hh-ocean" cx="38%" cy="36%" r="70%">
          <stop offset="0" stop-color="#5aa6e6" />
          <stop offset="1" stop-color="#1b4a8c" />
        </radialGradient>
        <radialGradient id="hh-night" cx="34%" cy="32%" r="78%">
          <stop offset="0.5" stop-color="#000" stop-opacity="0" />
          <stop offset="0.82" stop-color="#000" stop-opacity="0.45" />
          <stop offset="1" stop-color="#000" stop-opacity="0.8" />
        </radialGradient>
        <clipPath id="hh-globe">
          <circle cx="100" cy="100" r="92" />
        </clipPath>
      </defs>
      <circle cx="100" cy="100" r="96" fill="#8cc4ff" opacity="0.25" />
      <circle cx="100" cy="100" r="92" fill="url(#hh-ocean)" />
      <g clip-path="url(#hh-globe)">
        <path
          fill="#4f9a52"
          d="M48 40c14-10 34-8 42 2 6 8-2 16 6 24 7 7 20 4 22 14 2 9-10 14-18 13-10-1-12 8-20 10-11 3-16-8-14-17 2-8-8-10-14-16-8-8-14-20-4-30z"
        />
        <path
          fill="#438a47"
          d="M120 112c10-6 26-2 32 8 6 9 2 22-6 30-7 8-6 20-16 22-9 2-12-8-12-17 0-8-8-12-8-22 0-9 3-16 10-21z"
        />
        <path fill="#5aa65a" d="M140 46c8-3 18 0 20 7 2 6-5 9-11 9-7 0-15-2-15-8 0-3 2-6 6-8z" />
        <path fill="#e9eef2" d="M70 186c10-8 50-10 64 0z" />
        <path
          fill="#fff"
          opacity="0.85"
          d="M2 92c22-10 40 4 62-4s38-12 58-4 40 8 76-2v14c-30 10-52 4-74-2s-34 6-58 4S22 92 2 106z"
        />
        <path fill="#fff" opacity="0.6" d="M60 140c14-6 30 0 44-4s24-2 30 2c-12 6-26 2-40 6s-26 2-34-4z" />
        <path fill="#fff" opacity="0.55" d="M96 30c10-4 22-2 30 2-8 4-20 4-30-2z" />
        <rect x="0" y="0" width="200" height="200" fill="url(#hh-night)" />
      </g>
    </svg>
  );
}

/** Two rolling regolith layers with craters, a domed habitat, a mast and a lit window. */
export function Horizon() {
  return (
    <svg class="horizon" viewBox="0 0 300 140" preserveAspectRatio="xMidYMax slice" aria-hidden="true">
      <path class="far" d="M0 78C40 64 70 72 104 66S170 52 210 62 270 70 300 60V140H0Z" />
      <ellipse class="crater" cx="62" cy="78" rx="12" ry="2.5" />
      <ellipse class="crater" cx="244" cy="70" rx="9" ry="2" />
      <ellipse class="crater" cx="150" cy="66" rx="6" ry="1.5" />
      <path class="near" d="M0 106C50 94 100 104 160 100S250 92 300 98V140H0Z" />
      <ellipse class="crater" cx="40" cy="116" rx="18" ry="4" />
      <ellipse class="crater" cx="110" cy="128" rx="11" ry="2.8" />
      <ellipse class="crater" cx="272" cy="122" rx="14" ry="3.2" />
      <line class="metal" x1="232" y1="98" x2="232" y2="58" />
      <line class="metal" x1="226" y1="66" x2="238" y2="66" />
      <circle class="lamp" cx="232" cy="56" r="2.4" />
      <path class="dome" d="M150 101a14 11 0 0 1 28 0z" />
      <path class="dome" d="M172 101a26 21 0 0 1 52 0z" />
      <rect class="lamp" x="194" y="88" width="8" height="5" rx="1" />
    </svg>
  );
}
