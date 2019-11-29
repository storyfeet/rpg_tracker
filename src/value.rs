use crate::error::{ActionError, ParseError};
use crate::expr::Expr;
use crate::prev_iter::LineCounter;
use crate::token::Token;
use std::collections::BTreeMap;
use crate::proto::{Proto,ProtoP};

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
    Proto(Proto),
    None,
}


#[derive(Debug, Clone)]
pub enum SetPath {
    Ok(Option<Value>),
    Proto(Proto),
    Err,
}

impl Value {
    pub fn tree()->Self{
        Value::Tree(BTreeMap::new())
    }
    //Error in this case is the Proto value, so follow that pointer
    pub fn get_path<'a>(&'a self, mut pp: ProtoP) -> GotPath<'a> {
        match pp.next() {
            None => GotPath::Val(self),
            Some(p) => match self {
                Value::Tree(mp) => match mp.get(p) {
                    Some(ch) => return ch.get_path(pp),
                    None => match mp.get("proto") {
                        Some(Value::Str(s)) => {
                            let mut res = Proto::new(s);
                            res.extend(pp);
                            GotPath::Proto(res)
                        }
                        Some(_)|None => return GotPath::None,
                    },
                },
                _ => GotPath::None,
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
                Value::Tree(ref mut mp) =>{
                    if let Some(ch)= mp.get_mut(p) {
                        return ch.get_path_mut(pp)
                    }
                    None
                },
                _ => None
            },
        }
    }

    pub fn set_at_path<'a >(&'a mut self, mut pp:ProtoP, mut v: Value) -> SetPath {
        if pp.remaining() == 1{
            if let Value::Tree(t) = self{
                let rv = t.insert(pp.next().unwrap().to_string(),v);
                return SetPath::Ok(rv);
            }
            return SetPath::Err;
        }

        match pp.next() {
            None => {
                std::mem::swap(self,&mut v);
                SetPath::Ok(Some(v))
            }
            Some(p) => match self {
                Value::Tree(mp) => match mp.get_mut(p) {
                    Some(ch) => return ch.set_at_path(pp,v),

                    None => match mp.get("proto") {
                        Some(Value::Str(s)) => 
                                SetPath::Proto(Proto::new(s)),
                        Some(_)|None => return SetPath::Err,
                    },
                },
                _ => SetPath::Err,
            },
        }
    }

    fn try_add(self, rhs: Value) -> Result<Value, ActionError> {
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
            Ex(a) => match rhs {
                Ex(b) => Ok(Ex(a - b)),
                _ => Err(ActionError::new("Can only sub Ex from Ex")),
            },
            Str(_) => Err(ActionError::new("Cannot subtract from string")),
            List(a) => match rhs {
                Str(b) => Ok(List(a.into_iter().filter(|x| x != &b).collect())),

                List(b) => Ok(List(a.into_iter().filter(|x| !b.contains(&x)).collect())),
                _=>Err(ActionError::new("Cannot subtract non str/list from List"))
            },
            Tree(mut t)=>match rhs{
                Str(s)=>{
                    t.remove(&s);
                    Ok(Tree(t))
                }
                _=>Err(ActionError::new("Can only sub str from tree"))
                
            }
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
