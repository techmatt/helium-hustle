//! Spare threads for the oracle's rollouts (137). A grid (`axes`, `experiment`) runs
//! its games on `--threads` workers; once the queue holds fewer games than workers,
//! a finished worker's thread is lent to the grid's pool, and a look's independent
//! rollouts ([`map`]) borrow what is free in their own grid's pool. A grid's workers
//! plus what they borrow never pass its `--threads`, and results come back in item
//! order, so play is the same with or without spare threads.

use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

thread_local! {
    /// The pool of the grid this thread is a worker of ([`Lend::enter`]).
    static POOL: RefCell<Option<Arc<AtomicUsize>>> = const { RefCell::new(None) };
}

/// One grid's spare threads.
pub struct Lend {
    spare: Arc<AtomicUsize>,
}

/// While held, [`map`] on this thread borrows from its grid's pool.
pub struct Enter(());

impl Drop for Enter {
    fn drop(&mut self) {
        POOL.with(|p| *p.borrow_mut() = None);
    }
}

impl Lend {
    /// A pool lending `n` threads at once (workers the grid has no game for).
    pub fn new(n: usize) -> Lend {
        Lend { spare: Arc::new(AtomicUsize::new(n)) }
    }

    /// Lends one more: a worker that found the queue empty.
    pub fn give(&self) {
        self.spare.fetch_add(1, Ordering::SeqCst);
    }

    /// Marks the calling thread a worker of this grid (call at each worker's start).
    pub fn enter(&self) -> Enter {
        POOL.with(|p| *p.borrow_mut() = Some(self.spare.clone()));
        Enter(())
    }
}

/// Threads taken from a pool, returned when dropped (a panicking rollout included).
struct Borrowed<'a> {
    spare: &'a AtomicUsize,
    n: usize,
}

impl Drop for Borrowed<'_> {
    fn drop(&mut self) {
        self.spare.fetch_add(self.n, Ordering::SeqCst);
    }
}

/// Takes up to `want` threads from `spare`.
fn borrow(spare: &AtomicUsize, want: usize) -> Borrowed<'_> {
    let mut have = spare.load(Ordering::SeqCst);
    loop {
        let n = have.min(want);
        if n == 0 {
            return Borrowed { spare, n: 0 };
        }
        match spare.compare_exchange(have, have - n, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => return Borrowed { spare, n },
            Err(now) => have = now,
        }
    }
}

/// `f` over `items`, in their order, on this thread plus any spare ones of its grid
/// (none outside a grid's worker: then in turn, here).
pub fn map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let Some(spare) = POOL.with(|p| p.borrow().clone()) else {
        return items.iter().map(f).collect();
    };
    let lent = borrow(&spare, items.len().saturating_sub(1));
    if lent.n == 0 {
        return items.iter().map(f).collect();
    }
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<R>>> = Mutex::new(items.iter().map(|_| None).collect());
    let work = || loop {
        let i = next.fetch_add(1, Ordering::SeqCst);
        let Some(x) = items.get(i) else { break };
        let r = f(x);
        out.lock().unwrap_or_else(|e| e.into_inner())[i] = Some(r);
    };
    std::thread::scope(|sc| {
        for _ in 0..lent.n {
            sc.spawn(work);
        }
        work();
    });
    drop(lent);
    out.into_inner().unwrap_or_else(|e| e.into_inner()).into_iter().map(|r| r.expect("every item ran")).collect()
}
