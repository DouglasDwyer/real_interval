//! `RealInterval` provides an `f32`-backed continuous interval type for ergonomic
//! interval manipulation. Scalar operations, arithmetic operations, and set operations
//! on intervals are all supported. The following is a simple example of how to use
//! intervals:
//! 
//! ```
//! # use real_interval::*;
//! let interval = RealInterval::min_max(-1.0, 2.0);
//! let shifted_interval = interval + 0.5;
//! let expanded_interval = RealInterval::min_max(-2.0, 3.0) * interval;
//! 
//! assert_eq!(RealInterval::min_max(-0.5, 2.5), shifted_interval);
//! assert_eq!(RealInterval::min_max(-4.0, 6.0), expanded_interval);
//! 
//! let and_interval = interval & shifted_interval;
//! let or_interval = interval | shifted_interval;
//! 
//! assert_eq!(Some(RealInterval::min_max(-0.5, 2.0)), and_interval);
//! assert_eq!(RealInterval::min_max(-1.0, 2.5), or_interval);
//! ```

#![deny(warnings)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::ops::*;

/// Represents a closed range on the real numbers.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RealInterval {
    /// The starting value of this range.
    pub min: f32,
    /// The ending value of this range.
    pub max: f32
}

impl RealInterval {
    /// Creates a new interval from the given minimum and maximum.
    /// The maximum must be at least as big as the minimum, or this will panic.
    pub fn min_max(min: f32, max: f32) -> Self {
        assert!(min <= max);
        Self { min, max }
    }

    /// Creates a range that contains a single point.
    pub fn point(value: f32) -> Self {
        let min = value;
        let max = value;

        Self { min, max }
    }

    /// Creates a range from a point and extents around the point.
    pub fn point_extents(value: f32, half_extent: f32) -> Self {
        assert!(half_extent >= 0.0, "Extent {half_extent} was less than 0");

        let min = value - half_extent;
        let max = value + half_extent;
        Self { min, max }
    }

    /// Determines whether the provided value lies within this interval.
    pub fn contains(self, value: f32) -> bool {
        self.min <= value && value <= self.max
    }

    /// Takes the absolute value of this range.
    pub fn abs(self) -> Self {
        if self.min < 0.0 && 0.0 <= self.max {
            let min = 0.0;
            let max = self.max.max(self.min.abs());
            Self { min, max }
        }
        else {
            let a = self.min.abs();
            let b = self.max.abs();
            let min = a.min(b);
            let max = a.max(b);

            Self { min, max }
        }
    }

    /// Provides the length of this interval.
    pub fn len(&self) -> f32 {
        self.max - self.min
    }

    /// Applies the minimum function between two ranges.
    pub fn min(self, rhs: Self) -> Self {
        let min = self.min.min(rhs.min);
        let max = self.max.min(rhs.max);

        Self { min, max }
    }

    /// Applies the maximum function between two ranges.
    pub fn max(self, rhs: Self) -> Self {
        let min = self.min.max(rhs.min);
        let max = self.max.max(rhs.max);

        Self { min, max }
    }

    /// Applies a scalar minimum to this range.
    pub fn minf(self, value: f32) -> Self {
        let min = self.min.min(value);
        let max = self.max.min(value);

        Self { min, max }
    }

    /// Applies a scalar maximum to this range.
    pub fn maxf(self, value: f32) -> Self {
        let min = self.min.max(value);
        let max = self.max.max(value);

        Self { min, max }
    }

    /// Applies a scalar power to this range.
    /// The interval must be non-negative.
    pub fn powf(self, value: f32) -> Self {
        let min;
        let max;

        assert!(self.min >= 0.0);

        if value > 0.0 {
            min = self.min.powf(value);
            max = self.max.powf(value);
        }
        else {
            min = self.max.powf(value);
            max = self.min.powf(value);
        }

        Self { min, max }
    }

    /// Applies an integral scalar power to this range.
    pub fn powi(self, value: i32) -> Self {
        let min;
        let max;

        if value % 2 == 0 {
            if self.min < 0.0 && 0.0 <= self.max {
                min = 0.0;
                max = self.min.powi(value).max(self.max.powi(value));
            }
            else if 0.0 <= self.min {
                min = self.min.powi(value);
                max = self.max.powi(value);
            }
            else {
                min = self.max.powi(value);
                max = self.min.powi(value);
            }
        }
        else {
            min = self.min.powi(value);
            max = self.max.powi(value);
        }

        Self { min, max }
    }

    /// Multiplies this range by the provided integer power of 2.
    pub fn mul_pow2(self, value: i32) -> Option<Self> {
        (Self::verify_ldexp(self.min, value) && Self::verify_ldexp(self.max, value))
            .then(|| Self::mul_pow2_unchecked(self, value))
    }

    /// Multiplies this range by the provided integer power of 2 without overflow checking.
    /// This function is safe, but in the case where the exponent of either minimum or maximum
    /// has an overflow, the result is not specified.
    pub fn mul_pow2_unchecked(self, value: i32) -> Self {
        debug_assert!(Self::verify_ldexp(self.min, value) && Self::verify_ldexp(self.max, value), "Power-of-2 multiply caused overflow.");

        let min = Self::ldexp(self.min, value);
        let max = Self::ldexp(self.max, value);

        Self { min, max }
    }

    /// Rounds this range to the nearest whole number.
    pub fn round(self) -> Self {
        let min = self.min.round();
        let max = self.max.round();

        Self { min, max }
    }

    /// Verifies that multiplying the given float by the provided
    /// integer power of two will not cause an overflow or underflow.
    fn verify_ldexp(a: f32, exp: i32) -> bool {
        let bits = a.to_bits();
        let exponent = ((bits >> 23) & 0xff) as i32;
        
        if exp > 0 {
            exp <= 255 && exponent + exp <= 255
        }
        else {
            exponent + exp >= 0
        }
    }

    /// Multiples the provided float by the given positive integer power of two.
    /// If the exponent becomes too small or large, the float's
    /// underlying representation may overflow.
    fn ldexp(a: f32, exp: i32) -> f32 {
        f32::from_bits(a.to_bits() + ((exp << 23) as u32))
    }
}

impl BitAnd for RealInterval {
    type Output = Option<Self>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let min = f32::max(self.min, rhs.min);
        let max = f32::min(self.max, rhs.max);

        (min <= max).then_some(Self { min, max })
    }
}

impl BitOr for RealInterval {
    type Output = RealInterval;

    fn bitor(self, rhs: Self) -> Self::Output {
        let min = self.min.min(rhs.min);
        let max = self.max.max(rhs.max);

        Self { min, max }
    }
}

impl Add<f32> for RealInterval {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        let min = self.min + rhs;
        let max = self.max + rhs;
        Self { min, max }
    }
}

impl Sub<f32> for RealInterval {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        let min = self.min - rhs;
        let max = self.max - rhs;
        Self { min, max }
    }
}

impl Mul<f32> for RealInterval {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        if rhs >= 0.0 {
            let min = rhs * self.min;
            let max = rhs * self.max;
            Self { min, max }
        }
        else {
            let min = rhs * self.max;
            let max = rhs * self.min;
            Self { min, max }
        }
    }
}

impl Mul<RealInterval> for f32 {
    type Output = RealInterval;

    fn mul(self, rhs: RealInterval) -> Self::Output {
        if self >= 0.0 {
            let min = self * rhs.min;
            let max = self * rhs.max;
            RealInterval { min, max }
        }
        else {
            let min = self * rhs.max;
            let max = self * rhs.min;
            RealInterval { min, max }
        }
    }
}

impl Add for RealInterval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let min = self.min + rhs.min;
        let max = self.max + rhs.max;
        Self { min, max }
    }
}

impl Sub for RealInterval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let min = self.min - rhs.max;
        let max = self.max - rhs.min;
        Self { min, max }
    }
}

impl Mul for RealInterval {
    type Output = Self;

    fn mul(self, rhs: RealInterval) -> Self::Output {
        (self.min * rhs) | (self.max * rhs)
    }
}

impl Neg for RealInterval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let min = -self.max;
        let max = -self.min;
        Self { min, max }
    }
}

impl std::fmt::Display for RealInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}, {}]", self.min, self.max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition_and_sets() {
        let interval = RealInterval::min_max(-1.0, 2.0);
        let shifted_interval = interval + 0.5;
        let expanded_interval = RealInterval::min_max(-2.0, 3.0) * interval;

        assert_eq!(RealInterval::min_max(-0.5, 2.5), shifted_interval);
        assert_eq!(RealInterval::min_max(-4.0, 6.0), expanded_interval);

        let and_interval = interval & shifted_interval;
        let or_interval = interval | shifted_interval;

        assert_eq!(Some(RealInterval::min_max(-0.5, 2.0)), and_interval);
        assert_eq!(RealInterval::min_max(-1.0, 2.5), or_interval);
    }
}