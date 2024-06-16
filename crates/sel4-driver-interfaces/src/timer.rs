//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::time::Duration;

use crate::HandleInterrupt;

pub trait ErrorType {
    type Error: fmt::Debug;
}

pub trait Clock: ErrorType {
    fn get_time(&mut self) -> Result<Duration, Self::Error>;
}

pub trait Timers: Clock {
    type TimerLayout;

    type Timer;

    fn timer_layout(&mut self) -> Result<Self::TimerLayout, Self::Error>;

    fn set_timeout_on(&mut self, timer: Self::Timer, relative: Duration)
        -> Result<(), Self::Error>;

    fn clear_timeout_on(&mut self, timer: Self::Timer) -> Result<(), Self::Error>;
}

pub trait Timer: Clock {
    fn set_timeout(&mut self, relative: Duration) -> Result<(), Self::Error>;

    fn clear_timeout(&mut self) -> Result<(), Self::Error>;
}

impl<T: Timer> Timers for T {
    type TimerLayout = ();

    type Timer = ();

    fn timer_layout(&mut self) -> Result<Self::TimerLayout, Self::Error> {
        Ok(())
    }

    fn set_timeout_on(
        &mut self,
        _timer: Self::Timer,
        relative: Duration,
    ) -> Result<(), Self::Error> {
        self.set_timeout(relative)
    }

    fn clear_timeout_on(&mut self, _timer: Self::Timer) -> Result<(), Self::Error> {
        self.clear_timeout()
    }
}

pub struct NumTimers(pub usize);

pub struct DefaultTimer<T>(pub T);

impl<T: ErrorType> ErrorType for DefaultTimer<T> {
    type Error = T::Error;
}

impl<T: Clock> Clock for DefaultTimer<T> {
    fn get_time(&mut self) -> Result<Duration, Self::Error> {
        self.0.get_time()
    }
}

impl<T: Timers<Timer: Default>> Timer for DefaultTimer<T> {
    fn set_timeout(&mut self, relative: Duration) -> Result<(), Self::Error> {
        self.0.set_timeout_on(Default::default(), relative)
    }

    fn clear_timeout(&mut self) -> Result<(), Self::Error> {
        self.0.clear_timeout_on(Default::default())
    }
}

pub struct SingleTimer<T>(pub T);

impl<T: ErrorType> ErrorType for SingleTimer<T> {
    type Error = T::Error;
}

impl<T: Clock> Clock for SingleTimer<T> {
    fn get_time(&mut self) -> Result<Duration, Self::Error> {
        self.0.get_time()
    }
}

impl<T: Timer> Timers for SingleTimer<T> {
    type TimerLayout = NumTimers;

    type Timer = usize;

    fn timer_layout(&mut self) -> Result<Self::TimerLayout, Self::Error> {
        Ok(NumTimers(1))
    }

    fn set_timeout_on(
        &mut self,
        _timer: Self::Timer,
        relative: Duration,
    ) -> Result<(), Self::Error> {
        self.0.set_timeout(relative)
    }

    fn clear_timeout_on(&mut self, _timer: Self::Timer) -> Result<(), Self::Error> {
        self.0.clear_timeout()
    }
}

impl<T: HandleInterrupt> HandleInterrupt for SingleTimer<T> {
    fn handle_interrupt(&mut self) {
        self.0.handle_interrupt()
    }
}
