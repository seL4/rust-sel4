#![no_std]
#![feature(error_in_core)]
#![feature(lazy_cell)]
#![feature(thread_local)]

extern crate alloc;

use alloc::rc::{Rc, Weak};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::{LazyCell, RefCell};
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};

use futures::future::Future;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use futures::task::{waker_ref, ArcWake};
use futures::task::{Context, Poll};
use futures::task::{FutureObj, LocalFutureObj, LocalSpawn, Spawn, SpawnError};

mod enter;

#[derive(Debug)]
pub struct LocalPool {
    pool: FuturesUnordered<LocalFutureObj<'static, ()>>,
    incoming: Rc<Incoming>,
}

#[derive(Clone, Debug)]
pub struct LocalSpawner {
    incoming: Weak<Incoming>,
}

type Incoming = RefCell<Vec<LocalFutureObj<'static, ()>>>;

struct ThreadNotify {
    woken: AtomicBool,
}

impl ThreadNotify {
    fn new() -> Self {
        Self {
            woken: AtomicBool::new(false),
        }
    }

    fn wake(&self) {
        self.woken.store(true, Ordering::Release);
    }

    #[allow(dead_code)]
    fn woken(&self) -> bool {
        self.woken.load(Ordering::Acquire)
    }

    fn take_wakeup(&self) -> bool {
        self.woken.swap(false, Ordering::Acquire)
    }
}

#[thread_local]
static CURRENT_THREAD_NOTIFY: LazyCell<Arc<ThreadNotify>> =
    LazyCell::new(|| Arc::new(ThreadNotify::new()));

impl ArcWake for ThreadNotify {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        ThreadNotify::wake(arc_self);
    }
}

fn run_executor_until_stalled<T, F: FnMut(&mut Context<'_>) -> Poll<T>>(mut f: F) -> Poll<T> {
    let _enter =
        enter::enter().expect("cannot execute `LocalPool` executor from within another executor");

    let waker = waker_ref(&CURRENT_THREAD_NOTIFY);
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(t) = f(&mut cx) {
            return Poll::Ready(t);
        }

        if !CURRENT_THREAD_NOTIFY.take_wakeup() {
            return Poll::Pending;
        }
    }
}

impl LocalPool {
    /// Create a new, empty pool of tasks.
    pub fn new() -> Self {
        Self {
            pool: FuturesUnordered::new(),
            incoming: Default::default(),
        }
    }

    /// Get a clonable handle to the pool as a [`Spawn`].
    pub fn spawner(&self) -> LocalSpawner {
        LocalSpawner {
            incoming: Rc::downgrade(&self.incoming),
        }
    }

    pub fn run_all_until_stalled(&mut self) -> Poll<()> {
        run_executor_until_stalled(|cx| self.poll_pool(cx))
    }

    pub fn run_until_stalled<F: Future>(&mut self, mut future: Pin<&mut F>) -> Poll<F::Output> {
        run_executor_until_stalled(|cx| {
            {
                // if our main task is done, so are we
                let result = future.as_mut().poll(cx);
                if let Poll::Ready(output) = result {
                    return Poll::Ready(output);
                }
            }

            let _ = self.poll_pool(cx);
            Poll::Pending
        })
    }

    /// Poll `self.pool`, re-filling it with any newly-spawned tasks.
    /// Repeat until either the pool is empty, or it returns `Pending`.
    ///
    /// Returns `Ready` if the pool was empty, and `Pending` otherwise.
    ///
    /// NOTE: the pool may call `wake`, so `Pending` doesn't necessarily
    /// mean that the pool can't make progress.
    fn poll_pool(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        loop {
            self.drain_incoming();

            let pool_ret = self.pool.poll_next_unpin(cx);

            // We queued up some new tasks; add them and poll again.
            if !self.incoming.borrow().is_empty() {
                continue;
            }

            match pool_ret {
                Poll::Ready(Some(())) => continue,
                Poll::Ready(None) => return Poll::Ready(()),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    /// Empty the incoming queue of newly-spawned tasks.
    fn drain_incoming(&mut self) {
        let mut incoming = self.incoming.borrow_mut();
        for task in incoming.drain(..) {
            self.pool.push(task)
        }
    }
}

impl Default for LocalPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Run a future to completion on the current thread.
///
/// This function will block the caller until the given future has completed.
///
/// Use a [`LocalPool`](LocalPool) if you need finer-grained control over
/// spawned tasks.
pub fn run_until_stalled<F: Future>(mut future: Pin<&mut F>) -> Poll<F::Output> {
    run_executor_until_stalled(|cx| future.as_mut().poll(cx))
}

impl Spawn for LocalSpawner {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        if let Some(incoming) = self.incoming.upgrade() {
            incoming.borrow_mut().push(future.into());
            Ok(())
        } else {
            Err(SpawnError::shutdown())
        }
    }

    fn status(&self) -> Result<(), SpawnError> {
        if self.incoming.upgrade().is_some() {
            Ok(())
        } else {
            Err(SpawnError::shutdown())
        }
    }
}

impl LocalSpawn for LocalSpawner {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        if let Some(incoming) = self.incoming.upgrade() {
            incoming.borrow_mut().push(future);
            Ok(())
        } else {
            Err(SpawnError::shutdown())
        }
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        if self.incoming.upgrade().is_some() {
            Ok(())
        } else {
            Err(SpawnError::shutdown())
        }
    }
}
