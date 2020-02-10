use crate::action::Action;
use crate::error::{ActionError, LineError};
use crate::expr::Expr;
use crate::prev_iter::Backer;
use crate::prev_iter::LineCounter;
use crate::proto::{Proto, ProtoNode, ProtoP};
use crate::token::{TokPrev, Token};
use std::cmp::{Ordering, PartialOrd};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Num(i32),
    Str(String),
    List(Vec<Value>),
    Tree(BTreeMap<String, Value>),
    ExprDef(Box<Expr>),
    FuncDef(Vec<String>, Vec<Action>),
    Deref(Box<Value>),
    Proto(Proto),
}

pub enum SetResult {
    Ok(Option<Value>),
    Deref(Proto, Value),
    Err(ActionError),
}

impl From<usize> for Value {
    fn from(n: usize) -> Self {
        Value::Num(n as i32)
    }
}

impl Value {
    pub fn tree() -> Self {
        Value::Tree(BTreeMap::new())
    }

    pub fn str(s: &str) -> Self {
        Value::Str(s.to_string())
    }

    pub fn as_proto(&self) -> Result<&Proto, ActionError> {
        match self {
            Value::Proto(p) => Ok(p),
            _ => Err(ActionError::new("Cannot treat non proto as proto")),
        }
    }

    pub fn print(&self, depth: usize) -> String {
        use Value::*;
        let mut res = String::new();
        match self {
            Bool(b) => res.push_str(&b.to_string()),
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
            ExprDef(ex) => {
                res.push_str(&ex.print());
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
        if let Value::Proto(_) = self {
            return Some(self);
        };
        match pp.next() {
            None => Some(self),
            Some(p) => match self {
                Value::Tree(mp) => match mp.get(&p.as_string()) {
                    Some(ch) => return ch.get_path(pp),
                    None => None,
                },
                Value::List(l) => {
                    if let ProtoNode::Num(n) = p {
                        return match l.get(*n as usize) {
                            Some(ch) => return ch.get_path(pp),
                            None => None,
                        };
                    }
                    None
                }
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
                    if let Some(ch) = mp.get_mut(&p.as_string()) {
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
                    let rv = t.insert(pp.next().unwrap().as_string(), v);
                    return SetResult::Ok(rv);
                }
                //TODO cover list
                Value::Proto(p) => {
                    return SetResult::Deref(p.extend_new(pp), v);
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
                Value::Tree(mp) => match mp.get_mut(&p.as_string()) {
                    Some(ch) => return ch.set_at_path(pp, v),
                    None => {
                        let mut t = Value::tree();
                        let res = t.set_at_path(pp, v);
                        mp.insert(p.as_string(), t);
                        return res;
                    }
                },
                Value::Proto(p) => {
                    return SetResult::Deref(p.extend_new(pp), v);//TODO make sure the set result gets the deref somehow
                }
                _ => return SetResult::Err(ActionError::new("canot set child of non tree")),
            },
        }
    }

    /// Logic Or included
    pub fn try_add(self, rhs: Value) -> Result<Value, ActionError> {
        use Value::*;
        match self {
            Bool(a) => match rhs {
                Bool(b) => Ok(Bool(a || b)),
                _ => Err(ActionError::new("Bool only adds to bool is OR op")),
            },
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
            Proto(mut a) => match rhs {
                Proto(b) => {
                    a.extend(b.pp());
                    Ok(Proto(a))
                }
                e => Err(ActionError::new(&format!("proto cannot add {:?}", e))),
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
                v => Ok(List(a.into_iter().filter(|x| *x != v).collect())),
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
            Value::Bool(a) => match rhs {
                Value::Bool(b) => Ok(Value::Bool(a && b)),
                _ => Err(ActionError::new("Bool can only mul Bool as AND op")),
            },
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
                _ => Err(ActionError::new("No div on non ex")),
            },
            _ => Err(ActionError::new("No div on non ex")),
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

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        use Value::*;
        match self {
            //TODO allow other comparisons
            Num(a) => {
                if let Num(b) = other {
                    return a.partial_cmp(b);
                }
            }
            _ => return None,
        }
        None
    }
}

impl Value {
    pub fn func_def(it: &mut TokPrev) -> Result<Self, LineError> {
        //handle bracket
        match it.next().ok_or(it.eof())? {
            Token::Expr => {
                let ex = Expr::from_tokens(it)?;
                return Ok(Value::ExprDef(Box::new(ex)));
            }
            Token::Fn => {}
            e => return Err(it.err(&format!("Func def on notafunc {:?}", e))),
        }
        if it.next() != Some(Token::BracketO) {
            return Err(it.err("Func should start with '('"));
        }

        let mut params = Vec::new();
        //loop params
        while let Some(tk) = it.next() {
            match tk {
                Token::Ident(s) => params.push(s),
                Token::Comma | Token::Break => {}
                Token::BracketC => break,
                e => return Err(it.err(&format!("Ux {:?} in func params", e))),
            }
        }

        if it.next() != Some(Token::SquigleO) {
            return Err(it.err("Func has nothing to do"));
        }

        let mut actions = Vec::new();
        //loop actions
        while let Some(tk) = it.next() {
            match tk {
                Token::SquigleC => break,
                Token::Comma | Token::Break => {}
                _ => {
                    it.back();
                    actions.push(Action::from_tokens(it)?);
                }
            }
        }
        Ok(Value::FuncDef(params, actions))
    }

    pub fn from_tokens(it: &mut TokPrev) -> Result<Self, LineError> {
        match it.next() {
            None => Err(it.err("UX-EOF")),
            Some(Token::Qoth(s)) => Ok(Value::Str(s)),
            Some(Token::True) => Ok(Self::Bool(true)),
            Some(Token::False) => Ok(Self::Bool(false)),
            Some(Token::Num(n)) => Ok(Value::Num(n)),
            Some(Token::Expr) | Some(Token::Fn) => {
                it.back();
                Self::func_def(it)
            }
            Some(Token::SquareO) => {
                let mut rlist = Vec::new();
                while let Some(v) = it.next() {
                    match v {
                        Token::Comma | Token::Break => {}
                        Token::SquareC => return Ok(Value::List(rlist)),

                        _ => {
                            it.back();
                            rlist.push(Value::from_tokens(it)?)
                        }
                    }
                }

                Err(it.err("UX-EOF"))
            }
            v => Err(it.err(&format!("UX - {:?}", v))),
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
