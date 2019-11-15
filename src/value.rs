use std::collections::HashMap;
use std::fmt;

use serde::Serialize;

use crate::expr::EvalErrorMessage;

fn int_float_eq(i: i128, f: f64) -> bool {
    if f.trunc() == f {
        if std::i128::MIN as f64 <= f && f <= std::i128::MAX as f64 {
            (f as i128) == i
        } else {
            false
        }
    } else {
        false
    }
}

#[derive(Debug, Clone, PartialOrd, Serialize)]
pub enum Primitive {
    Boolean(bool),
    Integer(i128),
    Float(f64),
}
impl Primitive {
    /// Normal equals for other types, but approx for floats
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        if let Self::Float(f1) = self {
            if let Self::Float(f2) = other {
                return (f1 - f2).abs() < epsilon;
            }
        }
        self == other
    }

    /// Not
    pub fn not(&self) -> Result<Primitive, EvalErrorMessage> {
        use Primitive::*;
        Ok(match self {
            Boolean(a) => Boolean(!*a),
            a => {
                return Err(EvalErrorMessage::InvalidArgument(format!(
                    "Cannot (not {:?})",
                    a
                )));
            },
        })
    }

    /// And
    pub fn and(&self, other: &Self) -> Result<Primitive, EvalErrorMessage> {
        use Primitive::*;
        Ok(match (self, other) {
            (Boolean(a), Boolean(b)) => Boolean(*a && *b),
            (a, b) => {
                return Err(EvalErrorMessage::InvalidArgument(format!(
                    "Cannot (and {:?} {:?})",
                    a, b
                )));
            },
        })
    }

    /// Or
    pub fn or(&self, other: &Self) -> Result<Primitive, EvalErrorMessage> {
        use Primitive::*;
        Ok(match (self, other) {
            (Boolean(a), Boolean(b)) => Boolean(*a || *b),
            (a, b) => {
                return Err(EvalErrorMessage::InvalidArgument(format!(
                    "Cannot (or {:?} {:?})",
                    a, b
                )));
            },
        })
    }

    /// Add
    pub fn add(&self, other: &Self) -> Result<Primitive, EvalErrorMessage> {
        use Primitive::*;
        Ok(match (self, other) {
            (Integer(a), Integer(b)) => {
                Integer(a.checked_add(*b).ok_or(EvalErrorMessage::Overflow)?)
            },
            (Integer(a), Float(b)) | (Float(b), Integer(a)) => Float(*a as f64 + b),
            (Float(a), Float(b)) => Float(a + b),
            (a, b) => {
                return Err(EvalErrorMessage::InvalidArgument(format!(
                    "Cannot (add {:?} {:?})",
                    a, b
                )));
            },
        })
    }

    /// Multiply
    pub fn mul(&self, other: &Self) -> Result<Primitive, EvalErrorMessage> {
        use Primitive::*;
        Ok(match (self, other) {
            (Integer(a), Integer(b)) => {
                Integer(a.checked_mul(*b).ok_or(EvalErrorMessage::Overflow)?)
            },
            (Integer(a), Float(b)) | (Float(b), Integer(a)) => Float(*a as f64 * b),
            (Float(a), Float(b)) => Float(a + b),
            (a, b) => {
                return Err(EvalErrorMessage::InvalidArgument(format!(
                    "Cannot (mul {:?} {:?})",
                    a, b
                )));
            },
        })
    }
}
impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Boolean(s) => match other {
                Self::Boolean(o) => s == o,
                _ => false,
            },
            Self::Integer(s) => match other {
                Self::Integer(o) => s == o,
                Self::Float(o) => int_float_eq(*s, *o),
                _ => false,
            },
            Self::Float(s) => match other {
                Self::Integer(o) => int_float_eq(*o, *s),
                Self::Float(o) => s == o,
                _ => false,
            },
        }
    }
}
impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", match self {
            Self::Boolean(v) => v.to_string(),
            Self::Integer(v) => v.to_string(),
            Self::Float(v) => v.to_string(),
        })
    }
}

pub type Context = HashMap<String, Primitive>;
