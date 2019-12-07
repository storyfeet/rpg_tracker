use crate::action::Action;
use crate::error::{ActionError, LineError};
use crate::expr::Expr;
use crate::prev_iter::Backer;
use crate::prev_iter::LineCounter;
use crate::proto::{Proto, ProtoP};
use crate::token::{TokPrev, Token};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Num(i32),
    Str(String),
    List(Vec<Value>),
    Tree(BTreeMap<String, Value>),
    Proto(Proto),
    CallFunc(Proto,Vec<Value>),
    FuncDef(Vec<String>, Vec<Action>),
}

pub enum SetResult {
    Ok(Option<Value>),
    Deref(Proto, Value),
    Err(ActionError),
}

impl Value {
    pub fn tree() -> Self {
        Value::Tree(BTreeMap::new())
    }

    pub fn str(s: &str) -> Self {
        Value::Str(s.to_string())
    }

    pub fn print(&self, depth: usize) -> String {
        use Value::*;
        let mut res = String::new();
        match self {
            Num(n) => res.push_str(&n.to_string()),
            Tree(t) => {
                for (k, v) in t {
                    res.push('\n');
                    for _ in 0..depth {
                        res.push_str("  ");
                    }
                    res.push_str(k);
                    res.push(':');
                    res.push_str(&v.print(depth + 1));
                }
            }
            Proto(p) => {
                res.push_str(&p.to_string());
            }
            FuncDef(params, _) => {
                res.push_str(&format!("func{:?}", params));
            }
            List(l) => {
                res.push('[');
                for (i, v) in l.iter().enumerate() {
                    if i != 0 {
                        res.push(',');
                    }
                    res.push_str(&v.print(0));
                }

                res.push(']');
            }
            Str(s) => res.push_str(&format!("\"{}\"", s)),
            // v => res.push_str(&format!("{:?}", v)),
        }
        res
    }

    pub fn has_child(&self, s: &str) -> bool {
        match self {
            Value::Tree(t) => t.get(s).is_some(),
            _ => false,
        }
    }

    pub fn get_path<'a>(&'a self, pp: &mut ProtoP) -> Option<&'a Value> {
        //println!("value::get_path({:?},{:?})", self, pp);
        if let Value::Proto(_) = self {
            return Some(self);
        };
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

    pub fn set_at_path<'a>(&'a mut self, mut pp: ProtoP, mut v: Value) -> SetResult {
        if pp.remaining() == 1 {
            match self {
                Value::Tree(t) => {
                    let rv = t.insert(pp.next().unwrap().to_string(), v);
                    return SetResult::Ok(rv);
                }
                Value::Proto(p) => {
                    return SetResult::Deref(p.extend_new(pp).with_deref(1), v);
                }
                _ => return SetResult::Err(ActionError::new("Cannot set child of a non tree")),
            }
        }

        match pp.next() {
            None => {
                std::mem::swap(self, &mut v);
                SetResult::Ok(Some(v))
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
                Value::Proto(p) => {
                    return SetResult::Deref(p.extend_new(pp).with_deref(1), v);
                }
                _ => return SetResult::Err(ActionError::new("canot set child of non tree")),
            },
        }
    }

    pub fn try_add(self, rhs: Value) -> Result<Value, ActionError> {
        use Value::*;
        match self {
            Num(a) => match rhs {
                Num(b) => Ok(Num(a + b)),
                _ => Err(ActionError::new("Cannot add non num to num")),
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
            Num(a) => match rhs {
                Num(b) => Ok(Num(a - b)),
                _ => Err(ActionError::new("Can only sub num from num")),
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

    pub fn try_mul(self, rhs: Value) -> Result<Value, ActionError> {
        match self {
            Value::Num(a) => match rhs {
                Value::Num(b) => Ok(Value::Num(a * b)),
                _ => Err(ActionError::new("No mul on non num")),
            },
            _ => Err(ActionError::new("No mul on non num")),
        }
    }
    pub fn try_div(self, rhs: Value) -> Result<Value, ActionError> {
        match self {
            Value::Num(a) => match rhs {
                Value::Num(0) => Err(ActionError::new("Can't div by zero")),
                Value::Num(b) => Ok(Value::Num(a / b)),
                _ => Err(ActionError::new("No mul on non ex")),
            },
            _ => Err(ActionError::new("No mul on non ex")),
        }
    }
    pub fn try_neg(self) -> Result<Value, ActionError> {
        match self {
            Value::Num(v) => Ok(Value::Num(-v)),
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(ActionError::new("No neg non ex")),
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl Value {
    pub fn func_def(t: &mut TokPrev) -> Result<Self, LineError> {
        //handle bracket
        if t.next() != Some(Token::BOpen) {
            return Err(t.err("Func should start with '('"));
        }

        let mut params = Vec::new();
        //loop params
        while let Some(tk) = t.next() {
            match tk {
                Token::Ident(s) => params.push(s),
                Token::Comma | Token::Break => {}
                Token::BClose => break,
                e => return Err(t.err(&format!("Ux {:?} in func params", e))),
            }
        }

        if t.next() != Some(Token::SBOpen) {
            return Err(t.err("Func has nothing to do"));
        }

        let mut actions = Vec::new();
        //loop actions
        while let Some(tk) = t.next() {
            match tk {
                Token::SBClose => break,
                Token::Comma | Token::Break => {}
                _ => {
                    t.back();
                    actions.push(Action::from_tokens(t)?);
                }
            }
        }
        Ok(Value::FuncDef(params, actions))
    }



    pub fn from_tokens(it: &mut TokPrev) -> Result<Self, LineError> {
        match it.next() {
            None => Err(it.err("UX-EOF")),
            Some(Token::Qoth(s)) => Ok(Value::Str(s)),
            Some(Token::Ident(s)) => match s.as_ref() {
                "func" => return Self::func_def(it),
                "expr" => {
                    let ev = vec![Action::Expr(Expr::from_tokens(t)?)];
                    Ok(Value::FuncDef(Vec::new(), ev))
                }
                sv => Ok(Value::str(sv)),
            },
            Some(Token::Num(n)) => Ok(Value::Num(n)),
            Some(Token::Dollar) => {
                let pt = Value::Proto(Proto::from_tokens(t));
                match it.next() {
                    Some(Token::BOpen) => {
                        let mut params = Vec::new();
                        while let Some(tk) = it.next() {
                            match tk {
                                Token::BClose => {
                                    parts.push(Expr::Func(pt, params));
                                    break;
                                }
                                Token::Comma => {}
                                _ => {
                                    it.back();
                                    params.push(Value::from_tokens(it)?);
                                }
                            }
                        }
                    }
                    _ => {
                        it.back();
                        parts.push(Expr::Proto(pt));
                    }
            }
                
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
