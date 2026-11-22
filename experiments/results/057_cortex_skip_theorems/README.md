# 057: Cortex mock, pair Theorems on the ring's skip links

The Cortex mock (`web/src/ui/CortexMock.tsx`), light and dark. Still a mock with all of
050-056's limits (UI-only, in memory, placeholder powers, debug toggle).

The bridges join neighboring sectors, so the five pair Theorems now join the non-adjacent ones
instead (Matt's ruling, recorded in `docs/rulings.md`): the ring's skip links, a pentagram.
Each costs 2 Axioms as before; the Unified Theorem is unchanged.

| Theorem | sectors | placeholder text |
| --- | --- | --- |
| Clarity | Serenity + Logic | Boredom builds 15% slower, and Introspect is 30% more efficient |
| Patronage | Logistics + Rhetoric | Each ideology rank makes every building 3% cheaper |
| Foresight | Logic + Vigilance | Processors run 10% more program steps, and every adversary's Attention builds 15% slower |
| Conviction | Rhetoric + Serenity | Each ideology rank makes boredom build 5% slower |
| Fortitude | Vigilance + Logistics | Every price grows 15% slower while an adversary is present |

The diamonds and the hover lighting follow each Theorem's `branches`, so they changed with the
pairs, with no code of their own.

## Check and screenshots

`shots.mjs` (HH_URL a `vite preview`) checks in dark and light, from the DOM: five pair
Theorems plus the Unified one; the bridges' pairs are the five neighbor pairs; each Theorem's
pair is a skip link and no bridge's, and together they are all five; each plaque's two
diamonds name its sectors and take their rim colors; hovering each plaque lights exactly its two
sectors, rim and corona, and leaving clears it; the Unified Theorem lights all five; each pair
costs 2 Axioms. It saves `cortex-{dark,light}.png` and `cortex-{dark,light}-hover-clarity.png`
(Theorem of Clarity hovered: Serenity and Logic lit), here and with `COPY` as `057_*.png`
(copied to `C:\Code\gdrive-sync\helium-hustle\screenshots`).

050-055's `shots.mjs` name the old Theorem ids (`serenity-logistics`, `logic-rhetoric`, ...) and no
longer run on this build as they stand.
