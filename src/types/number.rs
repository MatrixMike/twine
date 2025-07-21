//! Numeric type implementation for Scheme
//!
//! This module implements the Number type that represents all numeric values
//! in Scheme. Currently supports floating-point numbers with special handling
//! for infinity and NaN values.

use std::str::FromStr;

/// Numeric value type for Scheme
///
/// Wraps f64 to provide Scheme-specific numeric behavior and validation.
/// Supports all standard numeric operations and special values like infinity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number(f64);

impl Number {
    /// Create a new Number from f64
    pub fn new(value: f64) -> Self {
        Number(value)
    }

    /// Get the underlying f64 value
    pub fn value(self) -> f64 {
        self.0
    }

    /// Check if this number represents an integer value
    pub fn is_integer(self) -> bool {
        self.0.fract() == 0.0 && self.0.is_finite()
    }

    /// Check if this number is finite (not infinite or NaN)
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    /// Check if this number is infinite (positive or negative)
    pub fn is_infinite(self) -> bool {
        self.0.is_infinite()
    }

    /// Check if this number is NaN (not a number)
    pub fn is_nan(self) -> bool {
        self.0.is_nan()
    }

    /// Check if this number is positive infinity
    pub fn is_positive_infinity(self) -> bool {
        self.0 == f64::INFINITY
    }

    /// Check if this number is negative infinity
    pub fn is_negative_infinity(self) -> bool {
        self.0 == f64::NEG_INFINITY
    }

    /// Positive infinity constant
    pub const INFINITY: Number = Number(f64::INFINITY);
    /// Negative infinity constant
    pub const NEG_INFINITY: Number = Number(f64::NEG_INFINITY);
    /// Not-a-number constant
    pub const NAN: Number = Number(f64::NAN);
    /// Zero constant
    pub const ZERO: Number = Number(0.0);
    /// One constant
    pub const ONE: Number = Number(1.0);
}

impl FromStr for Number {
    type Err = std::num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+inf.0" | "+infinity" => Ok(Number::INFINITY),
            "-inf.0" | "-infinity" => Ok(Number::NEG_INFINITY),
            "+nan.0" | "nan" => Ok(Number::NAN),
            _ => {
                let value = s.parse::<f64>()?;
                Ok(Number::new(value))
            }
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_positive_infinity() {
            write!(f, "+inf.0")
        } else if self.is_negative_infinity() {
            write!(f, "-inf.0")
        } else if self.is_nan() {
            write!(f, "+nan.0")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Number::new(value)
    }
}

impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Number::new(value as f64)
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Number::new(value as f64)
    }
}

impl From<Number> for f64 {
    fn from(number: Number) -> Self {
        number.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_parsing() {
        // Basic number parsing
        assert_eq!(Number::from_str("42").unwrap().value(), 42.0);
        assert_eq!(Number::from_str("-17").unwrap().value(), -17.0);
        assert_eq!(Number::from_str("3.14159").unwrap().value(), 3.14159);
        assert_eq!(Number::from_str("-2.718").unwrap().value(), -2.718);
        assert_eq!(Number::from_str("0").unwrap().value(), 0.0);
        assert_eq!(Number::from_str("-0").unwrap().value(), -0.0);

        // Scientific notation
        assert_eq!(Number::from_str("1e10").unwrap().value(), 1e10);
        assert_eq!(Number::from_str("-1.5e-3").unwrap().value(), -1.5e-3);
        assert_eq!(Number::from_str("6.022e23").unwrap().value(), 6.022e23);

        // Special values
        assert!(Number::from_str("+inf.0").unwrap().is_positive_infinity());
        assert!(
            Number::from_str("+infinity")
                .unwrap()
                .is_positive_infinity()
        );
        assert!(Number::from_str("-inf.0").unwrap().is_negative_infinity());
        assert!(
            Number::from_str("-infinity")
                .unwrap()
                .is_negative_infinity()
        );
        assert!(Number::from_str("+nan.0").unwrap().is_nan());
        assert!(Number::from_str("nan").unwrap().is_nan());

        // Parsing errors
        assert!(Number::from_str("not_a_number").is_err());
        assert!(Number::from_str("").is_err());
        assert!(Number::from_str("1.2.3").is_err());
    }

    #[test]
    fn test_number_formatting() {
        assert_eq!(format!("{}", Number::new(42.0)), "42");
        assert_eq!(format!("{}", Number::new(-17.5)), "-17.5");
        assert_eq!(format!("{}", Number::new(3.14159)), "3.14159");
        assert_eq!(format!("{}", Number::INFINITY), "+inf.0");
        assert_eq!(format!("{}", Number::NEG_INFINITY), "-inf.0");
        assert_eq!(format!("{}", Number::NAN), "+nan.0");
    }

    #[test]
    fn test_number_equality() {
        let num1 = Number::new(42.0);
        let num2 = Number::new(42.0);
        let num3 = Number::new(43.0);

        assert_eq!(num1, num2);
        assert_ne!(num1, num3);

        // Special case: NaN != NaN
        let nan1 = Number::NAN;
        let nan2 = Number::NAN;
        assert_ne!(nan1, nan2); // This is correct behavior for NaN
    }

    #[test]
    fn test_number_edge_cases() {
        // Test integer checking
        assert!(Number::new(42.0).is_integer());
        assert!(Number::new(-17.0).is_integer());
        assert!(!Number::new(3.14).is_integer());
        assert!(!Number::INFINITY.is_integer());
        assert!(!Number::NAN.is_integer());

        // Test finite checking
        assert!(Number::new(42.0).is_finite());
        assert!(Number::new(-17.5).is_finite());
        assert!(!Number::INFINITY.is_finite());
        assert!(!Number::NEG_INFINITY.is_finite());
        assert!(!Number::NAN.is_finite());

        // Test infinity checking
        assert!(!Number::new(42.0).is_infinite());
        assert!(Number::INFINITY.is_infinite());
        assert!(Number::NEG_INFINITY.is_infinite());
        assert!(!Number::NAN.is_infinite());

        // Test NaN checking
        assert!(!Number::new(42.0).is_nan());
        assert!(!Number::INFINITY.is_nan());
        assert!(Number::NAN.is_nan());
    }

    #[test]
    fn test_number_conversions() {
        // From various integer types
        let from_i32 = Number::from(42i32);
        assert_eq!(from_i32.value(), 42.0);

        let from_i64 = Number::from(-17i64);
        assert_eq!(from_i64.value(), -17.0);

        // From f64
        let from_f64 = Number::from(3.14159f64);
        assert_eq!(from_f64.value(), 3.14159);

        // To f64
        let to_f64: f64 = Number::new(2.718).into();
        assert_eq!(to_f64, 2.718);
    }
}
