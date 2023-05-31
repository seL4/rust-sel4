use core::hint;
use core::sync::atomic::{AtomicUsize, Ordering};

// ::spin::Barrier appears to use too tight of a loop

pub struct Barrier {
    num_threads: usize,
    count: AtomicUsize,
}

impl Barrier {
    pub const fn new(n: usize) -> Self {
        Self {
            num_threads: n,
            count: AtomicUsize::new(0),
        }
    }

    pub fn wait(&self) {
        let mut count = self.count.fetch_add(1, Ordering::SeqCst);
        while count < self.num_threads {
            hint::spin_loop();
            count = self.count.load(Ordering::SeqCst);
        }
    }
}
