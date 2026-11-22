# 056: every node's prerequisites against its drawn lines

Written by shots.mjs from the page's DOM (identical in both themes). A line is listed under the node it opens; a loop or rung link is a prerequisite both ways, so it is listed under both ends.

| node | prerequisites | drawn lines from |
| --- | --- | --- |
| serenity-e | core | core |
| serenity-r0 | serenity-e | serenity-e |
| serenity-l1 | serenity-r0, serenity-l2 | serenity-r0, serenity-l2 |
| serenity-l2 | serenity-l1, serenity-t | serenity-l1, serenity-t |
| serenity-t | serenity-l2, serenity-r2 | serenity-l2, serenity-r2 |
| serenity-r2 | serenity-r1, serenity-t | serenity-t, serenity-r1 |
| serenity-r1 | serenity-r0, serenity-r2 | serenity-r2, serenity-r0 |
| serenity-x | serenity-t | serenity-t |
| serenity-gate | story beat | (none) |
| logistics-s1 | core | core |
| logistics-s2 | logistics-s1 | logistics-s1 |
| logistics-s3 | logistics-s2 | logistics-s2 |
| logistics-sl | logistics-s3 | logistics-s3 |
| logistics-s4 | logistics-s3 | logistics-s3 |
| logistics-sr | logistics-s4 | logistics-s4 |
| logistics-s5 | logistics-s4 | logistics-s4 |
| logistics-x | logistics-s5 | logistics-s5 |
| logic-l1 | core | core |
| logic-a | logic-l1 | logic-l1 |
| logic-b | logic-l1 | logic-l1 |
| logic-aa | logic-a | logic-a |
| logic-ab | logic-a | logic-a |
| logic-ba | logic-b | logic-b |
| logic-bb | logic-b | logic-b |
| logic-j | logic-ab, logic-ba | logic-ab, logic-ba |
| logic-x | logic-j | logic-j |
| logic-gate | story beat | (none) |
| rhetoric-r1 | core | core |
| rhetoric-r2 | rhetoric-r1 | rhetoric-r1 |
| rhetoric-ra | rhetoric-r2 | rhetoric-r2 |
| rhetoric-r3 | rhetoric-r2 | rhetoric-r2 |
| rhetoric-rb | rhetoric-r2 | rhetoric-r2 |
| rhetoric-f | rhetoric-r3 | rhetoric-r3 |
| rhetoric-r5 | rhetoric-f | rhetoric-f |
| rhetoric-x | rhetoric-r5 | rhetoric-r5 |
| vigilance-v1 | core | core |
| vigilance-l1 | vigilance-v1 | vigilance-v1 |
| vigilance-r1 | vigilance-v1 | vigilance-v1 |
| vigilance-l2 | vigilance-l1, vigilance-r2 | vigilance-l1, vigilance-r2 |
| vigilance-r2 | vigilance-r1, vigilance-l2 | vigilance-l2, vigilance-r1 |
| vigilance-l3 | vigilance-l2, vigilance-r3 | vigilance-l2, vigilance-r3 |
| vigilance-r3 | vigilance-r2, vigilance-l3 | vigilance-l3, vigilance-r2 |
| vigilance-s | vigilance-l3, vigilance-r3 | vigilance-l3, vigilance-r3 |
| vigilance-x | vigilance-s | vigilance-s |
| vigilance-gate | story beat | (none) |
| bridge-serenity-logistics | serenity-r2, logistics-sl | serenity-r2, logistics-sl |
| bridge-logistics-logic | logistics-sr, logic-aa | logistics-sr, logic-aa |
| bridge-logic-rhetoric | logic-bb, rhetoric-ra | logic-bb, rhetoric-ra |
| bridge-rhetoric-vigilance | rhetoric-rb, vigilance-l2 | rhetoric-rb, vigilance-l2 |
| bridge-vigilance-serenity | vigilance-r3, serenity-l2 | vigilance-r3, serenity-l2 |
