use crate::error::ParseError;
use crate::expr::Expr;
use crate::prev_iter::LineCounter;
use crate::token::Token;
use std::ops::{Add, Sub};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Ex(Expr),
    Str(String),
    List(Vec<String>),
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Value) -> Value {
        use Value::*;
        match self {
            Null => rhs,
            Ex(mut a) => match rhs {
                Null => Ex(a),
                Ex(b) => Ex(a + b),
                Str(b) => Ex(a + Expr::Ident(b)),
                List(bb) => {
                    for b in bb {
                        a = a + Expr::Ident(b);
                    }
                    Ex(a)
                }
            },
            Str(mut a) => match rhs {
                Null => Str(a),
                Ex(b) => {
                    a.push_str(&format!("{:?}", b));
                    Str(a)
                }
                Str(b) => {
                    a.push_str(&b.to_string());
                    Str(a)
                }
                List(bb) => {
                    for b in bb {
                        a.push_str(&b)
                    }
                    Str(a)
                }
            },
            List(mut a) => match rhs {
                Null => List(a),
                Ex(b) => {
                    a.push(format!("{:?}", b));
                    List(a)
                }
                Str(b) => {
                    a.push(b);
                    List(a)
                }
                List(b) => {
                    a.extend(b);
                    List(a)
                }
            },
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Value) -> Value {
        use Value::*;
        match self {
            Null => Null,
            Ex(mut a) => match rhs {
                Null => Ex(a),
                Ex(b) => Ex(a - b),
                Str(b) => Ex(a - Expr::Ident(b)),
                List(bb) => {
                    for b in bb {
                        a = a - Expr::Ident(b);
                    }
                    Ex(a)
                }
            },
            Str(mut a) => match rhs {
                Null => Null,
                Ex(_) => Null,
                Str(b) => {
                    a.push_str(&b.to_string());
                    Str(a)
                }
                List(bb) => {
                    for b in bb {
                        a.push_str(&b)
                    }
                    Str(a)
                }
            },
            List(mut a) => match rhs {
                Null => Null,
                Ex(b) => {
                    a.push(format!("{:?}", b));
                    List(a)
                }
                Str(b) => {
                    a.push(b);

                    List(a)
                }
                List(b) => {
                    a.extend(b);
                    List(a)
                }
            },
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl From<Expr> for Value {
    fn from(e: Expr) -> Self {
        Value::Ex(e)
    }
}

impl Value {
    pub fn from_tokens<T: Iterator<Item = Token> + LineCounter>(
        t: &mut T,
    ) -> Result<Self, ParseError> {
        match t.next() {
            None => Err(t.err("UX-EOF")),
            Some(Token::BOpen) => Expr::from_tokens(t).map(|v| Value::Ex(v)),
            Some(Token::SBOpen) => {
                let mut rlist = Vec::new();
                while let Some(v) = t.next() {
                    match v {
                        Token::Qoth(s) | Token::Ident(s) => rlist.push(s),
                        Token::Num(n) => rlist.push(n.to_string()),
                        Token::SBClose => return Ok(Value::List(rlist)),
                        Token::Comma => {}
                        e => return Err(t.err(&format!("UX - {:?}", e))),
                    }
                }
                Err(t.err("UX-EOF"))
            }
            Some(Token::Ident(s)) | Some(Token::Qoth(s)) => Ok(Value::Str(s)),
            Some(Token::Num(n)) => Ok(Value::Ex(Expr::Num(n))),
            v => Err(t.err(&format!("UX - {:?}", v))),
        }
    }
}

pub struct VIter<'a> {
    v: &'a Value,
    n: usize,
}

impl<'a> Iterator for VIter<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        match self.v {
            Value::Str(s) => {
                if self.n > 0 {
                    return None;
                }
                self.n += 1;
                Some(s.to_string())
            }
            Value::List(l) => {
                let m = self.n;
                self.n += 1;
                l.get(m).map(|s| s.to_string())
            }
            _ => None,
        }
    }
}

impl<'a> IntoIterator for &'a Value {
    type IntoIter = VIter<'a>;
    type Item = String;
    fn into_iter(self) -> Self::IntoIter {
        VIter { v: self, n: 0 }
    }
}
