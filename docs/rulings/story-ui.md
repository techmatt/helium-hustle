# Rulings: story UI, the right panel and popups

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).
Split from [ui.md](ui.md) (art, cards, nav, bars) at 124.

- **Quest mechanics text (Matt, 2026-11-25; locked).** Mechanics never live in narrative
  text. A quest's `story` and `payoff` (and a one-off event's `summary`, `body` and
  `story`, amendment #3171) are narrative only: no instructions, no numbers. Goals, rewards
  and unlocks are their own content fields (a quest's `goal` and `reward`; a one-off's
  `effect`, where `{rate}` now lives) and always show apart, in the
  mechanics style (`Mechanics` in common.tsx; 108): the normal UI font one size smaller, each
  line led by a small muted label inline ("Goal:", "Reward:", "Effect:"), on a faint
  warm tan tint with no border. No monospace and no blue strip, anywhere. "Goal: …" reads the
  same done or not (no "Done" label: a ✓ on the card's title says it). A reward is a
  headline, not an inventory: only what the player actually gets (a granted command, panel,
  building, resource or lasting bonus; 1.3: "The Launch Full Pads command."), never research
  merely appearing on the market (players find it by the unread dots; Matt, 2026-11-26);
  secondary effects go unlisted; panels aren't rewards (Matt, 2026-11-28, 136: 1.6 opens the
  Timeline panel and the Experiment Lab with no Reward line; 1.1's line still names the
  Research panel, as he kept it); a quest that grants nothing shows no reward line at all (no
  "None."). No how-to hints (Matt, 2026-11-26): quests carry no `hints` and nothing shows
  a "How:" line, no handholding until playtests say what's needed. A one-off's Effect states
  only the actual change, tersely ("Boredom starts growing: +0.2/day."), never explaining it
  or telegraphing what comes later; the narrative carries the meaning (Matt, 2026-11-26). The
  popups (reward on the last payoff page, the next goal on the last page; a one-off's Effect)
  and Story cards use it. **A chain quest without a payoff** (Matt, 2026-11-27; 135) shows
  its Reward line at the end of the next quest's story popup, after the goal; its completion
  opens no popup of its own (1.1 First Light has none). The right panel is the exception: the main-quest box (amendment
  #3203: its title, its goal as one plain line in the UI font, and its progress), the
  side-quest lines and the Events lines carry no mechanics text. A content test
  keeps digits, `{rate}` and UI instructions out of narrative; new quests are written so.
- **Story cards (Matt, 2026-11-25; 108).** Every card in Story, main or side, starts
  collapsed: its picture, title (a ✓ once finished) and its story's first line with "…";
  an unfinished one also its progress. A click expands it to its story, one panel at a time
  (a pager under it; Matt, 2026-11-27, 135), and its Goal
  and Reward lines (the active beat's How lines too); a finished main beat's expanded card
  offers *Replay its story* (its modals). The spine is one unbroken line behind every dot,
  the active beat's included, with side quests branching off it.
- **Right panel (Matt, 2026-11-26; 112).** Top to bottom: the main-quest box (pinned); the program;
  **Events**, one line per one-off event fired this run (title and its one-line summary, no
  body or Effect) and per project completion this run ("Project complete."; 124), newest
  first, a click reopening its popup; **Side Quests** (Matt,
  2026-11-26), a fold like Events, "Side Quests (done/started)", folded showing the
  closest-to-done one's progress, the open ones only (title and progress bar; a click opens its
  popup: story, goal, reward), hidden when none is open; Recent Activity. Burnout and the boredom phases supersede each other
  in Events: only the latest fired this run shows (the engine view's `superseded`); Recent
  Activity still logs each. In the run Burnout fires in, Boredom Rising is skipped (no popup,
  no Events line; Matt, 2026-11-26, 113): it is the phase-2 notice from later runs. No
  "Completed" section and no side-quest banner: Story is the quest archive.
- **Popups (Matt, 2026-11-26; 112).** Every one-off event (Propellant discovery, Political
  Currents, Burnout, each boredom phase, each adversary's arrival: Matt, 2026-11-26, P7) and every main and side quest completion opens a
  popup when it fires (a chain quest without a payoff excepted, above) (the story modal, the clock held until its last button). A side quest's
  completion popup shows its reward, then it leaves the right panel. **"Hide popups for
  events you've already seen"**, a checkbox in Options (Matt, 2026-11-26), on by default, remembered per browser
  (`localStorage`): when on, an event whose popup this career has shown before (any earlier
  run; the set is kept in the save) doesn't pop up again but still lands in Events and Recent
  Activity. Quest completions pop up once a career anyway. **Project completions** (Matt,
  2026-11-27; 124): every one opens a popup as a quest completion does (never hidden as seen,
  not at a load, not under the autopilot): the project's name ("… complete") and card
  picture, a Reward line per reward and what that completion cost in total, in the
  mechanics style; it lists in Events.
- **Recent Activity filter (2026-11-25).** Checkboxes under the list show or hide each kind
  of line (shipments, quests, story, research, projects, achievements, adversaries, other),
  one only for a kind already in the feed. *Default (096):* remembered per browser
  (`localStorage`), not in the save.
- **Story art and title (2026-11-23, story_title_art_r2).** Each act opens with a painted
  side band (3:1, title at the left): Act 1 round 2's `side3` (launch pad and domes), Acts
  2-4 round 1's `side`. No sky, bleed, tall backdrop, painted spine or full-width takeover:
  Story stays in the middle column on the amber spine. Title: `split-glow` (HELIUM in ink,
  HUSTLE in glowing amber, a glowing rule), CSS on the theme tokens. Later acts' art shows
  only once the act is reached.
- **Quest pictures (2026-11-23).** Every quest may carry a picture (content, display only):
  medium above its story modal's paragraph, at the left column's foot while it is the active
  main quest (replacing the lunar-base scene, which stays the fallback for a quest without
  one), and as a thumbnail beside a beat's title in Story. Layouts must look right without
  one. Act 1's are Matt's picks from `quest_pictures_act1_r1` (080), 1.1's `earthrise`; a
  picture's stem is a word, never a quest number (099). A
  side quest shows its picture in its popups (112; the completion banner is gone).
