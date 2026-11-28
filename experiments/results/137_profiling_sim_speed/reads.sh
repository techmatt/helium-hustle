#!/bin/bash
# Paired timing reads, base then new back to back, each run from its own tree's root.
# usage: reads.sh <round>
R=$1
W=/c/Code/helium-hustle-worktrees/helium-hustle/137_profiling_sim_speed
B=$W/out/baseline/d9ab590
O=$W/out/137_profiling_sim_speed/reads_$R
LOG=/c/Code/gdrive-sync/helium-hustle/playlogs/2026-11-25_19-15-28_a86ed7_seed-0.jsonl
mkdir -p $O
t() { # name dir exe args...
  local name=$1 dir=$2 exe=$3; shift 3
  local s=$(date +%s.%N)
  (cd $dir && nice -n 10 $exe "$@" > $O/$name.txt 2>&1)
  local e=$(date +%s.%N)
  echo "$name $(python -c "print(round($e-$s,2))")" >> $O/times.txt
}
for side in base new; do
  if [ $side = base ]; then D=$B; X=$B/hh-sim.exe; else D=$W; X=$W/target/release/hh-sim.exe; fi
  t ladder_$side $D $X ladder
  t balance_$side $D $X balance
  t vshuman_$side $D $X vs-human $LOG
  t today_$side $D $X playlog $LOG --today
  t axes3_$side $D $X axes --skills skilled
done
echo READS-DONE >> $O/times.txt
