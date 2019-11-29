use crate::error::{ActionError, ParseError};
use crate::expr::Expr;
use crate::prev_iter::LineCounter;
use crate::token::Token;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Ex(Expr),
    Str(String),
    List(Vec<String>),
    Tree(BTreeMap<String, Value>),
}

#[derive(Debug)]
pub enum GotPath<'a> {
    Val(&'a Value),
    Proto(String),
    None,
}

impl Value {
    //Error in this case is the Proto value, so follow that pointer
    fn get_path<'a, T: Iterator<Item = &'a str>>(&'a self, mut i: T) -> GotPath<'a> {
        match i.next() {
            None => GotPath::Val(self),
            Some(p) => match self {
                Value::Tree(mp) => match mp.get(p) {
                    Some(ch) => return ch.get_path(i),
                    None => match mp.get("proto") {
                        Some(Value::Str(s)) => {
                            let res = s.clone();
                            for cs in i {
                                res.push('.');
                                res.push_str(&cs);
                            }
                            GotPath::Proto(res)
                        }
                        None => return GotPath::None,
                    },
                },
                _ => GotPath::None,
            },
        }
    }

    fn try_add(self, rhs: Value) -> Result<Value, ActionError> {
        use Value::*;
        match self {
            Ex(mut a) => match rhs {
                Ex(b) => Ok(Ex(a + b)),
                _ => Err(ActionError::new("Cannot add non Expression to Expression")),
            },
            Str(mut a) => match rhs {
                Str(b) => {
                    a.push_str(&b.to_string());
                    Ok(Str(a))
                }
                _ => Err(ActionError::new("Cannot add non str to str")),
            },
            List(mut a) => match rhs {
                Str(b) => {
                    a.push(b);
                    Ok(List(a))
                }
                List(b) => {
                    a.extend(b);
                    Ok(List(a))
                }
                _ => Err(ActionError::new("Can only add list or string to list")),
            },
            Tree(_) => Err(ActionError::new("Currently cannot add trees")),
        }
    }

    fn try_sub(self, rhs: Value) -> Result<Value, ActionError> {
        use Value::*;
        match self {
            Ex(mut a) => match rhs {
                Ex(b) => Ok(Ex(a - b)),
                _ => Err(ActionError::new("Can only sub Ex from Ex")),
            },
            Str(mut a) => Err(ActionError::new("Cannot subtract from string")),
            List(mut a) => match rhs {
                Str(b) => Ok(List(a.into_iter().filter(|x| x != &b).collect())),

                List(b) => Ok(List(a.into_iter().filter(|x| !b.contains(&x)).collect())),
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
