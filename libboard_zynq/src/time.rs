#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Milliseconds(pub u64);

impl core::ops::Add for Milliseconds {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Milliseconds(self.0 + rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Microseconds(pub u64);

impl core::ops::Add for Microseconds {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Microseconds(self.0 + rhs.0)
    }
}

pub trait TimeSource<U> {
    fn now(&self) -> U;
}
