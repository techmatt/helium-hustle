#!/bin/bash
# The 9-career skilled read (137's), base (main, built by `python scripts/baseline.py <rev>`)
# then new, back to back, two rounds. Edit W and B to rerun.
W=/c/Code/helium-hustle-worktrees/helium-hustle/138_sim_player_view_and_small_fixes
B=$W/out/baseline/1ef58e4
O=$W/out/138_sim_player_view_and_small_fixes
for round in 1 2; do
  for side in base new; do
    if [ $side = base ]; then D=$B; X=$B/hh-sim.exe; else D=$W; X=$W/target/release/hh-sim.exe; fi
    s=$(date +%s.%N)
    (cd $D && nice -n 10 $X axes --skills skilled --seeds 3 > $O/axes9_${side}_$round.txt 2>&1)
    e=$(date +%s.%N)
    echo "axes9_${side}_$round $(python -c "print(round($e-$s,1))")" >> $O/axes9_times.txt
  done
done
echo AXES9-DONE >> $O/axes9_times.txt
