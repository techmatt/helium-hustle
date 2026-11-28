#!/bin/bash
# The 9-career skilled read, new then base, back to back.
W=/c/Code/helium-hustle-worktrees/helium-hustle/137_profiling_sim_speed
B=$W/out/baseline/d9ab590
O=$W/out/137_profiling_sim_speed
for side in new base; do
  if [ $side = base ]; then D=$B; X=$B/hh-sim.exe; else D=$W; X=$W/target/release/hh-sim.exe; fi
  s=$(date +%s.%N)
  (cd $D && nice -n 10 $X axes --skills skilled --seeds 3 > $O/axes9_${side}2.txt 2>&1)
  e=$(date +%s.%N)
  echo "axes9_$side $(python -c "print(round($e-$s,1))")" >> $O/axes9_times.txt
done
echo AXES9-DONE >> $O/axes9_times.txt
