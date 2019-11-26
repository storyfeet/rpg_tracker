use crate::expr::Expr;
use std::ops::{Add,Sub};

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

impl Sub for Value{
    type Output = Value;
    fn sub(mut self,rhs:Value)->Value{
        use Value::*;
        match self {
            Null=>Null,
            Ex(mut a) => match rhs {
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
                Ex(b) => {
                    Null
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
