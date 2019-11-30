use crate::error::{ActionError, LineError};
use crate::expr::Expr;
use crate::parse::Action;
use crate::prev_iter::Backer;
use crate::prev_iter::LineCounter;
use crate::proto::{Proto, ProtoP};
use crate::token::Token;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Ex(Expr),
    Str(String),
    List(Vec<Value>),
    Tree(BTreeMap<String, Value>),
    Proto(Proto),
    Action(Box<Action>),
    FuncDef(Vec<String>, Vec<Value>),
}

impl Value {
    pub fn tree() -> Self {
        Value::Tree(BTreeMap::new())
    }
    pub fn num(n: i32) -> Self {
        Value::Ex(Expr::Num(n))
    }
    pub fn str(s: &str) -> Self {
        Value::Str(s.to_string())
    }
    //Error in this case is the Proto value, so follow that pointer
    pub fn get_path<'a>(&'a self, mut pp: ProtoP) -> Option<&'a Value> {
        match pp.next() {
            None => Some(self),
            Some(p) => match self {
                Value::Tree(mp) => match mp.get(p) {
                    Some(ch) => return ch.get_path(pp),
                    None => None,
                },
                _ => None,
            },
        }
    }

    ///lifetime issues means to get proto for get_mut you can't follow proto
    /// this is probably actually correct as a mutable property on
    /// an object shouldn't come from a proto
    pub fn get_path_mut<'a>(&'a mut self, mut pp: ProtoP) -> Option<&'a mut Value> {
        match pp.next() {
            None => Some(self),
            Some(p) => match self {
                Value::Tree(ref mut mp) => {
                    if let Some(ch) = mp.get_mut(p) {
                        return ch.get_path_mut(pp);
                    }
                    None
                }
                _ => None,
            },
        }
    }

    pub fn set_at_path<'a>(
        &'a mut self,
        mut pp: ProtoP,
        mut v: Value,
    ) -> Result<Option<Value>, ()> {
        if pp.remaining() == 1 {
            if let Value::Tree(t) = self {
                let rv = t.insert(pp.next().unwrap().to_string(), v);
                return Ok(rv);
            }
            return Err(());
        }

        match pp.next() {
            None => {
                std::mem::swap(self, &mut v);
                Ok(Some(v))
            }
            Some(p) => match self {
                Value::Tree(mp) => match mp.get_mut(p) {
                    Some(ch) => return ch.set_at_path(pp, v),
                    None => {
                        let mut t = Value::tree();
                        let res = t.set_at_path(pp, v);
                        mp.insert(p.to_string(), t);
                        return res;
                    }
                },
                _ => return Err(()),
            },
        }
    }

    pub fn try_add(self, rhs: Value) -> Result<Value, ActionError> {
        use Value::*;
        match self {
            Ex(a) => match rhs {
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
                List(b) => {
                    a.extend(b);
                    Ok(List(a))
                }
                b => {
                    a.push(b);
                    Ok(List(a))
                }
            },
            u => Err(ActionError::new(&format!("Add of {:?} not suppported", u))),
        }
    }

    pub fn try_sub(self, rhs: Value) -> Result<Value, ActionError> {
        use Value::*;
        match self {
            Ex(a) => match rhs {
                Ex(b) => Ok(Ex(a - b)),
                _ => Err(ActionError::new("Can only sub Ex from Ex")),
            },
            Str(_) => Err(ActionError::new("Cannot subtract from string")),
            List(a) => match rhs {
                List(b) => Ok(List(a.into_iter().filter(|x| !b.contains(&x)).collect())),
                _ => Err(ActionError::new("Cannot subtract non list from List")),
            },
            Tree(mut t) => match rhs {
                Str(s) => {
                    t.remove(&s);
                    Ok(Tree(t))
                }
                _ => Err(ActionError::new("Can only sub str from tree")),
            },
            u => Err(ActionError::new(&format!("Sub of {:?} not suppported", u))),
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
    pub fn from_tokens<T: Iterator<Item = Token> + LineCounter + Backer>(
        t: &mut T,
    ) -> Result<Self, LineError> {
        match t.next() {
            None => Err(t.err("UX-EOF")),
            Some(Token::Ident(s)) | Some(Token::Qoth(s)) => Ok(Value::Str(s)),
            Some(Token::Num(n)) => Ok(Value::Ex(Expr::Num(n))),
            Some(Token::Dollar) => Ok(Value::Proto(Proto::from_tokens(t))),
            Some(Token::BOpen) => Expr::from_tokens(t).map(|v| Value::Ex(v)),
            Some(Token::SBOpen) => {
                let mut rlist = Vec::new();
                while let Some(v) = t.next() {
                    match v {
                        Token::Comma => {}
                        Token::SBClose => return Ok(Value::List(rlist)),

                        _ => {
                            t.back();
                            rlist.push(Value::from_tokens(t)?)
                        }
                    }
                }

                Err(t.err("UX-EOF"))
            }
            v => Err(t.err(&format!("UX - {:?}", v))),
        }
    }
}

pub struct VIter<'a> {
    v: &'a Value,
    n: usize,
}

impl<'a> Iterator for VIter<'a> {
    type Item = Value;
    fn next(&mut self) -> Option<Self::Item> {
        match self.v {
            Value::Tree(_) => {
                return None;
            }
            Value::List(l) => {
                let m = self.n;
                self.n += 1;
                l.get(m).map(|v| v.clone())
            }
            _ => {
                if self.n > 0 {
                    return None;
                }
                self.n += 1;
                Some(self.v.clone())
            }
        }
    }
}

impl<'a> IntoIterator for &'a Value {
    type IntoIter = VIter<'a>;
    type Item = Value;
    fn into_iter(self) -> Self::IntoIter {
        VIter { v: self, n: 0 }
    }
}
