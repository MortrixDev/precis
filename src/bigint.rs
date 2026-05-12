#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigInt {
    limbs: Vec<u64>,
    negative: bool,
}

fn normalize(limbs: &mut Vec<u64>) {
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
}

impl BigInt {
    pub fn zero() -> Self {
        Self { limbs: vec![], negative: false }
    }

    pub fn one() -> Self {
        Self { limbs: vec![1], negative: false }
    }

    pub fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    pub fn is_negative(&self) -> bool {
        self.negative && !self.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        !self.negative && !self.is_zero()
    }

    pub fn negate(self) -> Self {
        if self.is_zero() {
            return self.clone();
        }
        Self { limbs: self.limbs.clone(), negative: !self.negative }
    }

    pub fn abs(self) -> Self {
        Self { limbs: self.limbs.clone(), negative: false }
    }
}

impl From<i64> for BigInt {
    fn from(n: i64) -> Self {
        if n == 0 {
            return Self::zero();
        }
        let negative = n < 0;
        let abs = n.unsigned_abs();
        Self { limbs: vec![abs], negative }
    }
}

impl From<u64> for BigInt {
    fn from(n: u64) -> Self {
        if n == 0 {
            return Self::zero();
        }
        Self { limbs: vec![n], negative: false }
    }
}

impl From<u32> for BigInt {
    fn from(n: u32) -> Self {
        Self::from(n as u64)
    }
}

impl From<i32> for BigInt {
    fn from(n: i32) -> Self {
        Self::from(n as i64)
    }
}
