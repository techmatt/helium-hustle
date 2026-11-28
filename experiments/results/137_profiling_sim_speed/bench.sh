#!/bin/bash
# Paired benches, base then new: view_bench, wake_bench (two rounds), one skilled case on one thread, vs-human --skilled.
W=/c/Code/helium-hustle-worktrees/helium-hustle/137_profiling_sim_speed
B=$W/out/baseline/d9ab590
O=$W/out/137_profiling_sim_speed/bench
LOG=/c/Code/gdrive-sync/helium-hustle/playlogs/2026-11-26_23-45-26_c07aa7_seed-0.jsonl
mkdir -p $O
cd $W
for r in 1 2; do
  for side in base new; do
    if [ $side = base ]; then E=$O/../base_bin; else E=$W/target/release/examples; fi
    nice -n 10 $E/view_bench.exe > $O/view_${side}_$r.txt 2>&1
    nice -n 10 $E/wake_bench.exe > $O/wake_${side}_$r.txt 2>&1
  done
done
for side in base new; do
  if [ $side = base ]; then D=$B; X=$B/hh-sim.exe; else D=$W; X=$W/target/release/hh-sim.exe; fi
  s=$(date +%s.%N)
  (cd $D && nice -n 10 $X axes --skills skilled --axes humanist --threads 1 > $O/axes1_$side.txt 2>&1)
  e=$(date +%s.%N); echo "axes1_$side $(python -c "print(round($e-$s,1))")" >> $O/times.txt
  s=$(date +%s.%N)
  (cd $D && nice -n 10 $X vs-human $LOG --skilled > $O/vsh_$side.txt 2>&1)
  e=$(date +%s.%N); echo "vshuman_skilled_$side $(python -c "print(round($e-$s,2))")" >> $O/times.txt
done
echo BENCH-DONE >> $O/times.txt
