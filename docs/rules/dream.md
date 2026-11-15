# Dream (module `dream`, variant `flat`)

What the Dream command does to boredom. Dream is a command, so it runs when a processor
reaches it in a program, in the programs step of [the day](README.md#the-day) (programs.md).
It is a module of its own so another Dream (such as Dream Echo, not built) can be added as a
variant.

## Rules

- Each Dream execution lowers boredom by `relief × bonus_factor ^ humanist rank`, not below
  zero. `bonus_factor` and the rank are the ideology module's (ideology.md); a negative
  Humanist rank makes the relief smaller than `relief`.
- Relief is flat: it doesn't fade with use, phase or run length.
- Dream is an effect command: it runs only when its full cost is on hand, otherwise it fails
  and does nothing (programs.md, *Running a command*). It is never skipped for caps (it
  produces nothing). It adds no boredom of its own.
- Its cost and unlock are content: 8 energy and 0.5 helium-3 per execution, unlocked by the
  Dream Protocols research.

## Actions

None of its own: Dream is queued in a program (`set_program`, programs.md).

## Params

| Param | Baseline | Meaning |
|---|---|---|
| `relief` | 2 | Boredom removed per Dream, before the Humanist bonus. |

## Content

`content/commands.json`: the `dream` command (its costs, `requires`, and effect `dream`).

## Port notes

- Conflict: Dream. Handoffs: flat, -2 boredom x 1.05^humanist rank. Data: effect
  `dream_echo`, and phase events say Dream effectiveness falls to 20/15/10%. Ruling: flat
  Dream (module `dream`, variant `flat`); the phase-event text is left as the Godot
  placeholder and changes nothing.
