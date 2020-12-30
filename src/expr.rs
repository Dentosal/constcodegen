use std::fmt;

use crate::functions::Functions;
use crate::value::{Context, Primitive};

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    string: String,
    start: usize,
    len: usize,
}
impl Location {
    pub fn new(s: &str, start: usize, len: usize) -> Self {
        Self {
            string: s.to_owned(),
            start,
            len,
        }
    }
    pub fn error_here(&self, message: EvalErrorMessage) -> EvalError {
        EvalError {
            location: self.clone(),
            message,
        }
    }
}
impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "  {}\n  {}{}",
            self.string,
            " ".repeat(self.start),
            "^".repeat(self.len)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct EvalError {
    pub location: Location,
    pub message: EvalErrorMessage,
}
impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use EvalErrorMessage::*;
        write!(
            f,
            "{}\n{}",
            match &self.message {
                InvalidChar(c) => format!("Invalid character {:?} for this position", c),
                EmptyExpression => "Empty expressions are not allowed".to_owned(),
                UnmatchedOpen => "Unmatched opening '('".to_owned(),
                UnmatchedClose => "Unmatched closing ')'".to_owned(),
                UnexpectedToken => "Unexpected token".to_owned(),
                CallNonSymbol => "Only functions can be called".to_owned(),
                UnknownSymbol(sym) => format!("Unknown symbol name {:?}", sym),
                UnknownFunction(name) => format!("Unknown function {:?}", name),
                ArgumentCount => "Function argument count incorrect".to_owned(),
                InvalidArgument(msg) => format!("Argument invalid: {}", msg),
                Overflow => "Overflow or underflow occurred".to_owned(),
            },
            self.location
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalErrorMessage {
    InvalidChar(char),
    EmptyExpression,
    UnmatchedOpen,
    UnmatchedClose,
    UnexpectedToken,
    CallNonSymbol,
    UnknownSymbol(String),
    UnknownFunction(String),
    ArgumentCount,
    InvalidArgument(String),
    Overflow,
}

#[derive(Debug, Clone)]
struct Token {
    location: Location,
    type_: TokenValue,
}
impl Token {
    pub fn error_here(&self, message: EvalErrorMessage) -> EvalError {
        self.location.error_here(message)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TokenValue {
    Literal(Primitive),
    Symbol(String),
    ExprOpen,
    ExprClose,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub location: Location,
    pub value: ExprValue,
}
impl Expr {
    pub fn error_here(&self, message: EvalErrorMessage) -> EvalError {
        self.location.error_here(message)
    }

    fn resolve_all(self, ctx: &Context) -> Result<Self, EvalError> {
        match self.value {
            ExprValue::Primitive(_) => Ok(self),
            ExprValue::Symbol(sym) => {
                if let Some(value) = ctx.get(&sym) {
                    Ok(Expr {
                        location: self.location,
                        value: ExprValue::Primitive(value.clone()),
                    })
                } else {
                    Err(EvalError {
                        location: self.location,
                        message: EvalErrorMessage::UnknownSymbol(sym.clone()),
                    })
                }
            },
            ExprValue::Call(sym, args) => Ok(Self {
                location: self.location,
                value: ExprValue::Call(
                    sym,
                    args.into_iter()
                        .map(|a| a.resolve_all(ctx))
                        .collect::<Result<Vec<Self>, EvalError>>()?,
                ),
            }),
        }
    }

    fn call_functions(self, fns: &Functions) -> Result<Self, EvalError> {
        if let ExprValue::Call(sym, args) = self.value {
            let args = args
                .into_iter()
                .map(|a| a.call_functions(fns))
                .collect::<Result<Vec<Self>, EvalError>>()?;

            if let Some(fn_) = fns.get(&sym) {
                fn_(self.location, args)
            } else {
                Err(EvalError {
                    location: self.location,
                    message: EvalErrorMessage::UnknownFunction(sym),
                })
            }
        } else {
            Ok(self)
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprValue {
    Primitive(Primitive),
    Symbol(String),
    Call(String, Vec<Expr>),
}

fn scan(text: &str) -> Result<Vec<Token>, EvalError> {
    lazy_static! {
        static ref RE_FLT: Regex = Regex::new(r"^[-+]?[0-9]+\.[0-9]+([eE][-+]?[0-9]+)?").unwrap();
        static ref RE_INT: Regex = Regex::new(r"^[-+]?[0-9_]*[0-9]").unwrap();
        static ref RE_RDX: Regex = Regex::new(r"^0(b|o|x)([0-9a-f_]*[0-9a-f])").unwrap();
        static ref RE_BLN: Regex = Regex::new(r"^(true|false)").unwrap();
        static ref RE_SYM: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*").unwrap();
    }

    let mut result = Vec::new();
    let mut offset: usize = 0;
    while offset < text.len() {
        if let Some(m) = RE_FLT.find(&text[offset..]) {
            result.push(Token {
                location: Location::new(text, offset, m.as_str().len()),
                type_: TokenValue::Literal(Primitive::Float(m.as_str().parse().unwrap())),
            });
            offset += m.as_str().len();
        } else if let Some(cap) = RE_RDX.captures(&text[offset..]) {
            let radix = cap.get(1);
            let number = cap.get(2).unwrap().as_str().replace("_", "");
            println!("{:?} => {:?}", cap.get(0), (&number, &radix));
            result.push(Token {
                location: Location::new(text, offset, cap.get(0).unwrap().as_str().len()),
                type_: TokenValue::Literal(Primitive::Integer(
                    i128::from_str_radix(
                        &number,
                        match radix.map(|m| m.as_str()) {
                            Some("b") => 2,
                            Some("o") => 8,
                            Some("x") => 16,
                            _ => panic!("Invalid radix"), // TODO: better error message
                        },
                    )
                    .expect("Integer parsing failed"), // TODO: better error message
                )),
            });
            offset += cap.get(0).unwrap().as_str().len();
        } else if let Some(cap) = RE_INT.captures(&text[offset..]) {
            result.push(Token {
                location: Location::new(text, offset, cap.get(0).unwrap().as_str().len()),
                type_: TokenValue::Literal(Primitive::Integer(
                    i128::from_str_radix(
                        cap.get(0).unwrap().as_str(), 10
                    )
                    .expect("Integer parsing failed"), // TODO: better error message
                )),
            });
            offset += cap.get(0).unwrap().as_str().len();
        } else if let Some(cap) = RE_BLN.captures(&text[offset..]) {
            let value_str = cap.get(0).unwrap().as_str();
            result.push(Token {
                location: Location::new(text, offset, value_str.len()),
                type_: TokenValue::Literal(Primitive::Boolean(value_str.parse().unwrap())),
            });
            offset += cap.get(0).unwrap().as_str().len();
        } else if let Some(m) = RE_SYM.find(&text[offset..]) {
            result.push(Token {
                location: Location::new(text, offset, m.as_str().len()),
                type_: TokenValue::Symbol(m.as_str().to_owned()),
            });
            offset += m.as_str().len();
        } else {
            match text[offset..].chars().nth(0).unwrap() {
                c if c.is_whitespace() => {
                    offset += 1;
                },
                '(' => {
                    result.push(Token {
                        location: Location::new(text, offset, 1),
                        type_: TokenValue::ExprOpen,
                    });
                    offset += 1;
                },
                ')' => {
                    result.push(Token {
                        location: Location::new(text, offset, 1),
                        type_: TokenValue::ExprClose,
                    });
                    offset += 1;
                },
                other => {
                    return Err(EvalError {
                        location: Location::new(text, offset, 1),
                        message: EvalErrorMessage::InvalidChar(other),
                    });
                },
            }
        }
    }

    Ok(result)
}

/// Parse S-expression
fn parse_expr(tokens: Vec<Token>) -> Result<Expr, EvalError> {
    type Level = u32;

    assert!(tokens.len() > 1 && tokens[0].type_ == TokenValue::ExprOpen);
    let mut level: Level = 1;
    let mut index: usize = 1;
    let mut buffer: Vec<(Level, Expr, usize)> = Vec::new();
    while index < tokens.len() {
        let location = tokens[index].location.clone();
        match tokens[index].type_.clone() {
            TokenValue::Literal(val) => buffer.push((
                level,
                Expr {
                    location,
                    value: ExprValue::Primitive(val),
                },
                index,
            )),
            TokenValue::Symbol(sym) => buffer.push((
                level,
                Expr {
                    location,
                    value: ExprValue::Symbol(sym),
                },
                index,
            )),
            TokenValue::ExprOpen => {
                level += 1;
            },
            TokenValue::ExprClose => {
                level -= 1;
                if level == 0 && index + 1 < tokens.len() {
                    return Err(tokens[index + 1].error_here(EvalErrorMessage::UnexpectedToken));
                }
                let mut buf_index: usize = buffer.len();
                while buf_index > 0 {
                    buf_index -= 1;
                    if buffer[buf_index].0 != level + 1 {
                        buf_index += 1;
                        break;
                    }
                }

                let last_tok_index = buffer[buf_index].2;
                let mut expr_iter = buffer.drain(buf_index..).map(|(_, e, i)| (e, i));
                if let Some((function, fn_tok_index)) = expr_iter.next() {
                    if let ExprValue::Symbol(fn_sym) = function.value {
                        let args = expr_iter.map(|(e, _)| e).collect();
                        buffer.push((
                            level,
                            Expr {
                                location: function.location,
                                value: ExprValue::Call(fn_sym, args),
                            },
                            fn_tok_index,
                        ));
                    } else {
                        return Err(
                            tokens[fn_tok_index].error_here(EvalErrorMessage::CallNonSymbol)
                        );
                    }
                } else {
                    let token = &tokens[last_tok_index];
                    return Err(token.error_here(EvalErrorMessage::EmptyExpression));
                }
            },
        }

        index += 1;
    }

    if level > 0 {
        Err(tokens[0].error_here(EvalErrorMessage::UnexpectedToken))
    } else {
        assert_eq!(buffer.len(), 1);
        Ok(buffer.remove(0).1)
    }
}

/// Parse tokens to either S-expression or atom
fn parse(tokens: Vec<Token>) -> Result<Expr, EvalError> {
    if tokens.is_empty() {
        Err(EvalError {
            location: Location::new("", 0, 0),
            message: EvalErrorMessage::EmptyExpression,
        })
    } else if tokens.len() == 1 {
        match tokens[0].type_.clone() {
            TokenValue::Literal(val) => Ok(Expr {
                location: tokens[0].location.clone(),
                value: ExprValue::Primitive(val),
            }),
            TokenValue::Symbol(sym) => Ok(Expr {
                location: tokens[0].location.clone(),
                value: ExprValue::Symbol(sym),
            }),
            TokenValue::ExprOpen => Err(tokens[0].error_here(EvalErrorMessage::UnmatchedOpen)),
            TokenValue::ExprClose => Err(tokens[0].error_here(EvalErrorMessage::UnmatchedClose)),
        }
    } else {
        match tokens[0].type_.clone() {
            TokenValue::Literal(_) | TokenValue::Symbol(_) => {
                Err(tokens[1].error_here(EvalErrorMessage::UnexpectedToken))
            },
            TokenValue::ExprClose => Err(tokens[0].error_here(EvalErrorMessage::UnmatchedClose)),
            TokenValue::ExprOpen => parse_expr(tokens),
        }
    }
}

pub fn evaluate(text: &str, ctx: &Context, fns: &Functions) -> Result<Primitive, EvalError> {
    let expr = parse(scan(text)?)?.resolve_all(ctx)?.call_functions(fns)?;

    if let ExprValue::Primitive(p) = expr.value {
        Ok(p)
    } else {
        unimplemented!("{:?}", expr)
    }
}

#[cfg(test)]
mod test_expr {
    use crate::functions::Functions;
    use crate::value::{Context, Primitive};

    use super::evaluate;

    macro_rules! approx_eq {
        ($v1:expr, $v2:expr) => {{ $v1.approx_eq(&$v2, 0.01) }};
    }

    macro_rules! evaluate {
        ($s:expr) => {{ evaluate($s, &Context::new(), &Functions::default()) }};
    }

    #[test]
    fn test_eval_int() {
        for v in -10..=10 {
            assert_eq!(evaluate!(&v.to_string()), Ok(Primitive::Integer(v)));
        }

        assert_eq!(evaluate!("-0"), Ok(Primitive::Integer(0)));
        assert_eq!(evaluate!("+1"), Ok(Primitive::Integer(1)));
    }

    #[test]
    fn test_eval_hex_int() {
        assert_eq!(evaluate!("0x1"), Ok(Primitive::Integer(1)));
        assert_eq!(evaluate!("0xf00"), Ok(Primitive::Integer(0xf00)));
        assert_eq!(evaluate!("0xffff_8000_0000_0000"), Ok(Primitive::Integer(0xffff_8000_0000_0000)));
    }

    #[test]
    fn test_eval_float() {
        for v in -100..=100 {
            let f = f64::from(v);
            assert!(approx_eq!(
                evaluate!(&format!("{:.4}", f)).unwrap(),
                Primitive::Float(f)
            ));

            for q in 1..=100 {
                let f = f64::from(v) / f64::from(q);
                assert!(approx_eq!(
                    evaluate!(&format!("{:.4}", f)).unwrap(),
                    Primitive::Float(f)
                ));
            }
        }
    }

    #[test]
    fn test_eval_add() {
        assert_eq!(evaluate!("(add 1 2)"), Ok(Primitive::Integer(3)));

        assert_eq!(evaluate!("(add 1 2 3)"), Ok(Primitive::Integer(6)));

        assert_eq!(
            evaluate!("(add (add 1 2) (add 3 4))"),
            Ok(Primitive::Integer(10))
        );

        assert!(approx_eq!(
            evaluate!("(add 1.2 3.4)").unwrap(),
            Primitive::Float(4.6)
        ));
    }
}
