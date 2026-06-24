use crate::bigint::BigInt;

pub struct Real {
    numerator: BigInt,
    denominator: BigInt,
}

impl From<u64> for Real {
    fn from(n: u64) -> Self {
        Self { numerator: BigInt::from(n), denominator: BigInt::one() }
    }
}

impl From<i64> for Real {
    fn from(n: i64) -> Self {
        Self { numerator: BigInt::from(n), denominator: BigInt::one() }
    }
}

impl From<u32> for Real {
    fn from(n: u32) -> Self {
        Self::from(n as u64)
    }
}

impl From<i32> for Real {
    fn from(n: i32) -> Self {
        Self::from(n as i64)
    }
}

impl From<f64> for Real {
    fn from(n: f64) -> Self {
        assert!(n.is_finite(), "cannot convert NaN or infinity to Real");
        if n == 0.0 {
            return Self::from(0i64);
        }
        let bits = n.to_bits();
        let sign = bits >> 63;
        let biased_exp = ((bits >> 52) & 0x7FF) as i32;
        let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;

        let (significand, exponent) = if biased_exp == 0 {
            (mantissa, -1074i32)
        } else {
            (mantissa | (1u64 << 52), biased_exp - 1075)
        };

        let (numerator, denominator) = if exponent >= 0 {
            (BigInt::from(significand) << exponent as u32, BigInt::one())
        } else {
            (
                BigInt::from(significand),
                BigInt::one() << (-exponent) as u32,
            )
        };

        let numerator = if sign == 1 {
            numerator.negate()
        } else {
            numerator
        };
        let gcd = numerator.clone().gcd(denominator.clone());
        Self {
            numerator: numerator.div_exact(gcd.clone()).unwrap(),
            denominator: denominator.div_exact(gcd).unwrap(),
        }
    }
}

impl From<f32> for Real {
    fn from(n: f32) -> Self {
        Real::from(n as f64)
    }
}

impl std::fmt::Display for Real {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{} / {}", self.numerator, self.denominator);
    }
}

impl std::ops::Neg for Real {
    type Output = Real;

    fn neg(self) -> Real {
        Real {
            numerator: self.numerator.negate(),
            denominator: self.denominator,
        }
    }
}

impl std::ops::Add for Real {
    type Output = Real;

    fn add(self, rhs: Self) -> Real {
        let num = self.numerator.clone() * rhs.denominator.clone()
            + rhs.numerator.clone() * self.denominator.clone();
        let den = self.denominator * rhs.denominator;
        let gcd = num.clone().gcd(den.clone());
        Real {
            numerator: num.div_exact(gcd.clone()).unwrap(),
            denominator: den.div_exact(gcd).unwrap(),
        }
    }
}

impl std::ops::Sub for Real {
    type Output = Real;

    fn sub(self, rhs: Self) -> Real {
        self + (-rhs)
    }
}

impl std::ops::Mul for Real {
    type Output = Real;

    fn mul(self, rhs: Self) -> Self::Output {
        let num = self.numerator.clone() * rhs.numerator.clone();
        let den = self.denominator * rhs.denominator;
        let gcd = num.clone().gcd(den.clone());
        Real {
            numerator: num.div_exact(gcd.clone()).unwrap(),
            denominator: den.div_exact(gcd).unwrap(),
        }
    }
}
