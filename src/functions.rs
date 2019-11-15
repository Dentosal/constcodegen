use std::collections::HashMap;

use crate::expr::{EvalError, EvalErrorMessage, Expr, ExprValue, Location};
use crate::value::Primitive;

type R = Result<Expr, EvalError>;
type F = fn(Location, Vec<Expr>) -> R;

#[derive(Debug)]
pub struct Functions(HashMap<String, F>);
impl Functions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn default() -> Self {
        let mut result = Self::new();
        result.insert("not", f_not);
        result.insert("and", f_and);
        result.insert("or", f_or);
        result.insert("add", f_add);
        result.insert("mul", f_mul);
        result.insert("fract", f_fract);
        result
    }

    pub fn insert(&mut self, key: &str, value: F) {
        self.0.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&F> {
        self.0.get(key)
    }
}

macro_rules! check_argc_exact {
    ($c:expr; $location:expr, $args:expr) => {
        if $args.len() != $c {
            return Err(EvalError {
                location: $location,
                message: EvalErrorMessage::ArgumentCount,
            });
        }
    };
}

macro_rules! check_argc_min {
    ($minc:expr; $location:expr, $args:expr) => {
        if $args.len() < $minc {
            return Err(EvalError {
                location: $location,
                message: EvalErrorMessage::ArgumentCount,
            });
        }
    };
}

macro_rules! value {
    ($arg:expr) => {
        if let ExprValue::Primitive(p) = $arg.value.clone() {
            p
        } else {
            unreachable!("Calls and symbols should not exist anymore")
        }
    };
}

fn f_not(location: Location, args: Vec<Expr>) -> Result<Expr, EvalError> {
    check_argc_exact!(1; location, args);
    let acc = value!(args[0])
        .not()
        .map_err(|err| args[0].error_here(err))?;
    Ok(Expr {
        location,
        value: ExprValue::Primitive(acc),
    })
}

fn f_and(location: Location, args: Vec<Expr>) -> Result<Expr, EvalError> {
    check_argc_min!(1; location, args);
    let mut acc = Primitive::Boolean(true);
    for arg in args.into_iter() {
        acc = acc.and(&value!(arg)).map_err(|err| arg.error_here(err))?;
    }
    Ok(Expr {
        location,
        value: ExprValue::Primitive(acc),
    })
}

fn f_or(location: Location, args: Vec<Expr>) -> Result<Expr, EvalError> {
    check_argc_min!(1; location, args);
    let mut acc = Primitive::Boolean(false);
    for arg in args.into_iter() {
        acc = acc.or(&value!(arg)).map_err(|err| arg.error_here(err))?;
    }
    Ok(Expr {
        location,
        value: ExprValue::Primitive(acc),
    })
}

fn f_add(location: Location, args: Vec<Expr>) -> Result<Expr, EvalError> {
    check_argc_min!(2; location, args);
    let mut acc = value!(args[0]);
    for arg in args.into_iter().skip(1) {
        acc = acc.add(&value!(arg)).map_err(|message| EvalError {
            location: arg.location,
            message,
        })?;
    }
    Ok(Expr {
        location,
        value: ExprValue::Primitive(acc),
    })
}

fn f_mul(location: Location, args: Vec<Expr>) -> Result<Expr, EvalError> {
    check_argc_min!(2; location, args);
    let mut acc = value!(args[0]);
    for arg in args.into_iter().skip(1) {
        acc = acc.mul(&value!(arg)).map_err(|message| EvalError {
            location: arg.location,
            message,
        })?;
    }
    Ok(Expr {
        location,
        value: ExprValue::Primitive(acc),
    })
}

fn f_fract(location: Location, args: Vec<Expr>) -> Result<Expr, EvalError> {
    check_argc_exact!(1; location, args);
    if let Primitive::Float(p) = value!(args[0]) {
        Ok(Expr {
            location,
            value: ExprValue::Primitive(Primitive::Float(p.fract())),
        })
    } else {
        Err(args[0].error_here(EvalErrorMessage::InvalidArgument(
            "Only floats have fractional parts".to_owned(),
        )))
    }
}
