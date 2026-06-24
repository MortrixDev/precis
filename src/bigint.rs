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

    pub fn is_even(&self) -> bool {
        self.limbs.first().map_or(true, |l| l & 1 == 0)
    }

    pub fn div_exact(self, rhs: Self) -> Option<Self> {
        assert!(!rhs.is_zero(), "division by zero");
        let (q, r) = div_rem_magnitudes(&self.limbs, &rhs.limbs);
        if !r.is_empty() { return None; }
        let negative = self.negative != rhs.negative && !q.is_empty();
        Some(BigInt { limbs: q, negative })
    }

    pub fn gcd(self, other: Self) -> Self {
        let mut a = self.abs();
        let mut b = other.abs();
        if a.is_zero() { return b; }
        if b.is_zero() { return a; }
        let mut shift = 0u32;
        while a.is_even() && b.is_even() {
            a = a >> 1;
            b = b >> 1;
            shift += 1;
        }
        while a.is_even() {
            a = a >> 1;
        }
        loop {
            while b.is_even() {
                b = b >> 1;
            }
            if cmp_magnitude(&a.limbs, &b.limbs) == std::cmp::Ordering::Greater {
                std::mem::swap(&mut a, &mut b);
            }
            b = b - a.clone();
            if b.is_zero() {
                break;
            }
        }
        a << shift
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

impl std::fmt::Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }
        if self.negative {
            write!(f, "-")?;
        }
        let mut digits = Vec::new();
        let mut limbs = self.limbs.clone();

        while !limbs.is_empty() {
            let mut remainder = 0u128;
            for l in limbs.iter_mut().rev() {
                let val = remainder * (1u128 << 64) + *l as u128;
                *l = (val / 10) as u64;
                remainder = val % 10;
            }
            normalize(&mut limbs);
            digits.push(remainder as u8);
        }
        for d in digits.iter().rev() {
            write!(f, "{d}")?;
        }
        Ok(())
    }
}

fn cmp_magnitude(a: &[u64], b: &[u64]) -> std::cmp::Ordering {
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }
    for (av, bv) in a.iter().rev().zip(b.iter().rev()) {
        match av.cmp(bv) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

fn add_magnitudes(a: &[u64], b: &[u64]) -> Vec<u64> {
    let len = a.len().max(b.len());
    let mut result = Vec::with_capacity(len + 1);
    let mut carry = 0u128;
    for i in 0..len {
        let sum =
            a.get(i).copied().unwrap_or(0) as u128 + b.get(i).copied().unwrap_or(0) as u128 + carry;
        result.push(sum as u64);
        carry = sum >> 64;
    }
    if carry > 0 {
        result.push(carry as u64);
    }
    result
}

fn sub_magnitudes(a: &[u64], b: &[u64]) -> Vec<u64> {
    // requires a >= b in magnitude
    let mut result = Vec::with_capacity(a.len());
    let mut borrow = 0u64;
    for i in 0..a.len() {
        let av = a[i];
        let bv = b.get(i).copied().unwrap_or(0);
        let (diff, o1) = av.overflowing_sub(bv);
        let (diff, o2) = diff.overflowing_sub(borrow);
        result.push(diff);
        borrow = (o1 || o2) as u64;
    }
    normalize(&mut result);
    result
}

fn mul_magnitudes(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut result = vec![0u64; a.len() + b.len()];
    for (i, &av) in a.iter().enumerate() {
        let mut carry = 0u128;
        for (j, &bv) in b.iter().enumerate() {
            let prod = av as u128 * bv as u128 + result[i + j] as u128 + carry;
            result[i + j] = prod as u64;
            carry = prod >> 64;
        }
        result[i + b.len()] += carry as u64;
    }
    normalize(&mut result);
    result
}

fn shr_magnitude(limbs: &[u64], n: u32) -> Vec<u64> {
    if limbs.is_empty() || n == 0 { return limbs.to_vec(); }
    let word_shift = (n / 64) as usize;
    let bit_shift = n % 64;
    if word_shift >= limbs.len() { return vec![]; }
    let limbs = &limbs[word_shift..];
    let mut result: Vec<u64> = if bit_shift == 0 {
        limbs.to_vec()
    } else {
        (0..limbs.len())
            .map(|i| {
                let lo = limbs[i] >> bit_shift;
                let hi = limbs.get(i + 1).map_or(0, |&l| l << (64 - bit_shift));
                lo | hi
            })
            .collect()
    };
    normalize(&mut result);
    result
}

fn shl_magnitude(limbs: &[u64], n: u32) -> Vec<u64> {
    if limbs.is_empty() || n == 0 { return limbs.to_vec(); }
    let word_shift = (n / 64) as usize;
    let bit_shift = n % 64;
    let mut result = vec![0u64; limbs.len() + word_shift + 1];
    if bit_shift == 0 {
        result[word_shift..word_shift + limbs.len()].copy_from_slice(limbs);
    } else {
        for (i, &l) in limbs.iter().enumerate() {
            result[i + word_shift] |= l << bit_shift;
            result[i + word_shift + 1] |= l >> (64 - bit_shift);
        }
    }
    normalize(&mut result);
    result
}

fn div_rem_magnitudes(a: &[u64], b: &[u64]) -> (Vec<u64>, Vec<u64>) {
    assert!(!b.is_empty(), "division by zero");
    if a.is_empty() { return (vec![], vec![]); }
    match cmp_magnitude(a, b) {
        std::cmp::Ordering::Less => return (vec![], a.to_vec()),
        std::cmp::Ordering::Equal => return (vec![1], vec![]),
        _ => {}
    }
    let a_bits = a.len() as u32 * 64 - a.last().unwrap().leading_zeros();
    let b_bits = b.len() as u32 * 64 - b.last().unwrap().leading_zeros();
    let bit_diff = a_bits - b_bits;
    let mut quotient = vec![0u64; bit_diff as usize / 64 + 1];
    let mut remainder = a.to_vec();
    for i in (0..=bit_diff).rev() {
        let shifted = shl_magnitude(b, i);
        if cmp_magnitude(&remainder, &shifted) != std::cmp::Ordering::Less {
            remainder = sub_magnitudes(&remainder, &shifted);
            quotient[i as usize / 64] |= 1u64 << (i % 64);
        }
    }
    normalize(&mut quotient);
    normalize(&mut remainder);
    (quotient, remainder)
}

impl std::ops::Shr<u32> for BigInt {
    type Output = BigInt;

    fn shr(self, n: u32) -> BigInt {
        BigInt { limbs: shr_magnitude(&self.limbs, n), negative: self.negative }
    }
}

impl std::ops::Shl<u32> for BigInt {
    type Output = BigInt;

    fn shl(self, n: u32) -> BigInt {
        BigInt { limbs: shl_magnitude(&self.limbs, n), negative: self.negative }
    }
}

impl std::ops::Add for BigInt {
    type Output = BigInt;

    fn add(self, rhs: Self) -> Self::Output {
        if self.negative == rhs.negative {
            return BigInt {
                limbs: add_magnitudes(&self.limbs, &rhs.limbs),
                negative: self.negative,
            };
        }
        match cmp_magnitude(&self.limbs, &rhs.limbs) {
            std::cmp::Ordering::Equal => BigInt::zero(),
            std::cmp::Ordering::Greater => BigInt {
                limbs: sub_magnitudes(&self.limbs, &rhs.limbs),
                negative: self.negative,
            },
            std::cmp::Ordering::Less => BigInt {
                limbs: sub_magnitudes(&rhs.limbs, &self.limbs),
                negative: rhs.negative,
            },
        }
    }
}

impl std::ops::Sub for BigInt {
    type Output = BigInt;

    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs.negate()
    }
}

impl std::ops::Mul for BigInt {
    type Output = BigInt;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_zero() || rhs.is_zero() {
            return BigInt::zero();
        }
        BigInt {
            limbs: mul_magnitudes(&self.limbs, &rhs.limbs),
            negative: self.negative != rhs.negative,
        }
    }
}
