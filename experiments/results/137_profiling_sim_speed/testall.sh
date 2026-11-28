#!/bin/bash
# cargo test-all, base (its own target dir) and new, each warmed once, then timed base, new.
W=/c/Code/helium-hustle-worktrees/helium-hustle/137_profiling_sim_speed
B=$W/out/baseline/d9ab590
O=$W/out/137_profiling_sim_speed
export BT="C:/Code/helium-hustle-worktrees/helium-hustle/137_profiling_sim_speed/out/target-137-base"
run() { # side label
  local s=$(date +%s.%N)
  if [ $1 = base ]; then
    (cd $B && CARGO_TARGET_DIR="$BT" nice -n 10 cargo test-all -j 6 > $O/testall_$1_$2.log 2>&1)
  else
    (cd $W && nice -n 10 cargo test-all -j 6 > $O/testall_$1_$2.log 2>&1)
  fi
  local e=$(date +%s.%N)
  echo "testall_$1_$2 $(python -c "print(round($e-$s,1))") exit=$?" >> $O/testall_times.txt
}
run base warm
run new warm
run base timed
run new timed
echo TESTALL-PAIR-DONE >> $O/testall_times.txt
