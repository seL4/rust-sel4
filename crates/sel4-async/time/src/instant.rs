use core::ops::{Add, AddAssign, Sub, SubAssign};
use core::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    since_zero: Duration,
}

impl Instant {
    pub const ZERO: Self = Self::new(Duration::ZERO);

    pub const fn new(since_zero: Duration) -> Self {
        Self { since_zero }
    }

    pub const fn since_zero(&self) -> Duration {
        self.since_zero
    }

    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.since_zero.checked_sub(earlier.since_zero)
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.since_zero.checked_add(duration).map(Self::new)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.since_zero.checked_sub(duration).map(Self::new)
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        self.checked_add(other)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.checked_duration_since(other)
            .expect("overflow when instant from instant")
    }
}
