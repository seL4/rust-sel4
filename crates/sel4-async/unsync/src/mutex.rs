use core::cell::{RefCell, RefMut};
use core::ops::{Deref, DerefMut};

use async_unsync::semaphore::{Permit, Semaphore};

pub struct Mutex<T> {
    data: RefCell<T>, // remove need for unsafe
    semaphore: Semaphore,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: RefCell::new(data),
            semaphore: Semaphore::new(1),
        }
    }

    pub async fn lock(&self) -> Guard<'_, T> {
        let permit = self.semaphore.acquire().await.unwrap();
        let ref_mut = self.data.borrow_mut();
        Guard {
            ref_mut,
            _permit: permit,
        }
    }
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct Guard<'a, T> {
    // order matters, for drop order
    ref_mut: RefMut<'a, T>,
    _permit: Permit<'a>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.ref_mut
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ref_mut
    }
}
