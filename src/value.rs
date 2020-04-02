use crate::action::Action;
use crate::ecs_ish::{GenData, GenManager};
use crate::error::ActionError;
use crate::expr::Expr;
//use crate::scope::Scope;
use crate::proto::ProtoNode;
use std::cmp::{Ordering, PartialOrd};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Num(isize),
    Str(String),
    Ref(GenData),
    List(Vec<GenData>),
    Map(BTreeMap<ProtoNode, GenData>),
    ExprDef(Vec<String>, Expr),
    FuncDef(Vec<String>, Vec<Action>),
}

impl From<usize> for Value {
    fn from(n: usize) -> Self {
        Value::Num(n as isize)
    }
}

impl Value {
    pub fn map() -> Self {
        Value::Map(BTreeMap::new())
    }

    pub fn str(s: &str) -> Self {
        Value::Str(s.to_string())
    }

    pub fn print(&self, depth: usize, gm: &GenManager) -> String {
        use Value::*;
        let mut res = String::new();
        match self {
            Bool(b) => res.push_str(&b.to_string()),
            Num(n) => res.push_str(&n.to_string()),
            Map(t) => {
                for (k, vg) in t {
                    res.push('\n');
                    res.extend((0..depth).map(|_| ' '));
                    if let Some(v) = gm.get(vg) {
                        res.push_str(&k.as_string());
                        res.push(':');
                        res.push_str(&v.print(depth + 1, gm));
                    } else {
                        res.push_str("Freeed Pointer Err");
                    }
                }
            }
            ExprDef(_, ex) => {
                res.push_str(&ex.print());
            }
            FuncDef(params, _) => {
                res.push_str(&format!("func{:?}", params));
            }
            List(l) => {
                res.push('[');
                for (i, vg) in l.iter().enumerate() {
                    if i != 0 {
                        res.push(',');
                    }
                    if let Some(v) = gm.get(vg) {
                        res.push_str(&v.print(0, gm));
                    } else {
                        res.push_str("Freeed Pointer Err");
                    }
                }

                res.push(']');
            }
            Str(s) => res.push_str(&format!("\"{}\"", s)),

            Ref(vg) => {
                res.push('$');
                if let Some(v) = gm.get(vg) {
                    res.push_str(&v.print(0, gm));
                } else {
                    res.push_str("Freeed Pointer Err");
                }
            }
        }
        res
    }

    pub fn child_ref(&self, pn: &ProtoNode) -> Option<&GenData> {
        match self {
            Value::Map(t) => t.get(pn),
            Value::List(v) => {
                if let ProtoNode::Num(n) = pn {
                    v.get(*n)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Logic Or included
    pub fn try_add(self, rhs: Value, gm: &mut GenManager) -> Result<Value, ActionError> {
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
                _ => {
                    for i in a {
                        gm.drop_ref(i);
                    }
                    gm.drop(rhs);
                    Err(ActionError::new("Cannot add non list to list"))
                }
            },
            Map(mut ma) => {
                if let Map(mb) = rhs {
                    for (k, v) in mb {
                        if let Some(d) = ma.insert(k, v) {
                            gm.drop_ref(d)
                        }
                    }
                    Ok(Value::Map(ma))
                } else {
                    for (_, v) in ma {
                        gm.drop_ref(v);
                    }
                    gm.drop(rhs);
                    Err(ActionError::new("Cannot add non map to map"))
                }
            }
            //TODO Map + Map
            u => {
                let e = Err(ActionError::new(&format!("Add of {:?} not suppported", u)));
                gm.drop(u);
                gm.drop(rhs);
                e
            }
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
                Ref(v) => Ok(List(a.into_iter().filter(|x| *x != v).collect())),
                _ => unimplemented!(),
            },
            Map(mut t) => match rhs {
                Str(s) => {
                    t.remove(&ProtoNode::str(&s));
                    Ok(Map(t))
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

    pub fn gen_drop(self) -> Vec<GenData> {
        match self {
            Value::List(v) => v,
            Value::Map(m) => m.into_iter().map(|(_, v)| v).collect(),
            Value::Ref(r) => vec![r],
            _ => Vec::new(),
        }
    }

    pub fn clone_ignore_rc(&self) -> Value {
        match self {
            Value::Bool(b) => Value::Bool(*b),
            Value::Num(n) => Value::Num(*n),
            Value::Str(s) => Value::Str(s.clone()),
            Value::Ref(gd) => Value::Ref(gd.clone_ignore_gm()),
            Value::List(v) => Value::List(v.iter().map(|gd| gd.clone_ignore_gm()).collect()),
            Value::Map(m) => {
                let mut res = BTreeMap::new();
                for (k, v) in m {
                    res.insert(k.clone(), v.clone_ignore_gm());
                }
                Value::Map(res)
            }
            Value::ExprDef(p, e) => Value::ExprDef(p.clone(), e.clone()),
            Value::FuncDef(p, a) => Value::FuncDef(p.clone(), a.clone()),
        }
    }

    pub fn rc_up(&self, gm: &mut GenManager) {
        match self {
            Value::List(l) => {
                for i in l {
                    gm.inc_rc(i);
                }
            }
            Value::Map(m) => {
                for v in m.values() {
                    gm.inc_rc(v);
                }
            }
            Value::Ref(r) => {
                gm.inc_rc(r);
            }
            _ => {}
        }
    }

    pub fn clone_shallow(&self, gm: &mut GenManager) -> Value {
        let res = self.clone_ignore_rc();
        res.rc_up(gm);
        res
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
