#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Milliseconds(pub u64);

impl core::ops::Add for Milliseconds {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Milliseconds(self.0 + rhs.0)
    }
}
