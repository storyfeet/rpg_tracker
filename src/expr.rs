use crate::action::{AcResult, AcReturn, Action};
use crate::error::ActionError;
//use crate::prev_iter::Backer;
//use crate::prev_iter::LineCounter;
use crate::proto::{Proto, ProtoNode};
use crate::scope::Scope;
use crate::value::Value;
use gobble::err::ECode;
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq)]
pub enum EvReturn {
    Func,
    Expr,
    No,
    //  Break(usize),
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Op {
    Add,
    Sub,
    Div,
    Mul,
    Greater,
    Less,
    Dot,
    Equal,
}

impl FromStr for Op {
    type Err = ECode;
    fn from_str(s: &str) -> Result<Self, ECode> {
        use Op::*;
        Ok(match s {
            "+" => Add,
            "-" => Sub,
            "/" => Div,
            "*" => Mul,
            ">" => Greater,
            "<" => Less,
            "." => Dot,
            "==" => Equal,
            _ => return Err(ECode::Never("not a legit operator")),
        })
    }
}

impl Op {
    pub fn rank(&self) -> i32 {
        use Op::*;
        match self {
            Dot => 0,
            Add => 10,
            Sub => 9,
            Mul => 8,
            Div => 7,
            Greater => 6,
            Less => 5,
            Equal => 4,
        }
    }

    pub fn to_str(&self) -> &str {
        use Op::*;
        match self {
            Dot => ".",
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            Greater => ">",
            Less => "<",
            Equal => "==",
        }
    }

    pub fn eval(&self, a: &Expr, b: &Expr, sc: &mut Scope) -> Result<Value, ActionError> {
        match self {
            Op::Add => a.eval(sc)?.try_add(b.eval(sc)?, sc.gm_mut()),
            Op::Sub => a.eval(sc)?.try_sub(b.eval(sc)?),
            Op::Mul => a.eval(sc)?.try_mul(b.eval(sc)?),
            Op::Div => a.eval(sc)?.try_div(b.eval(sc)?),
            Op::Greater => Ok(Value::Bool(a.eval(sc)? > b.eval(sc)?)),
            Op::Less => Ok(Value::Bool(a.eval(sc)? < b.eval(sc)?)),
            Op::Equal => Ok(Value::Bool(a.eval(sc)? == b.eval(sc)?)),
            Op::Dot => Err(ActionError::new("Could not appy .= operation")),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct MapItem {
    pub k: String,
    pub v: Expr,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Bool(bool),
    Num(isize),
    Str(String),
    Ident(String),
    Oper(Op, Box<Expr>, Box<Expr>),
    Bracket(Box<Expr>),
    Neg(Box<Expr>),
    DotStart(Box<Expr>),
    Rooted(Box<Expr>),
    Deref(Box<Expr>),
    List(Vec<Expr>),
    Map(Vec<MapItem>),
    If(Box<Expr>, Vec<Action>, Vec<Action>),
    Call(Box<Expr>, Vec<Expr>),
    ExprDef(Vec<String>, Box<Expr>),
    FuncDef(Vec<String>, Vec<Action>),
}

impl Expr {
    pub fn neg(e: Expr) -> Self {
        Expr::Neg(Box::new(e))
    }

    pub fn eval_path(&self, sc: &mut Scope) -> Result<Proto, ActionError> {
        use Expr::*;
        Ok(match self {
            Num(n) => Proto::num(*n as usize),
            Str(s) => Proto::str(s),
            Ident(s) => Proto::str(s),
            Rooted(e) => e.eval_path(sc)?.rooted(),
            Deref(e) => Proto::dr().extend_new(e.eval_path(sc)?.pp()),
            DotStart(e) => e.eval_path(sc)?.dot(),
            Oper(Op::Dot, a, b) => a.eval_path(sc)?.extend_new(b.eval_path(sc)?.pp()),
            ot => match ot.eval(sc)? {
                Value::Num(n) => Proto::num(n as usize),
                Value::Str(s) => Proto::str(&s),
                ov => {
                    sc.gm_mut().drop(ov);
                    return Err(ActionError::new("Could not treat as proto"));
                }
            },
        })
    }

    pub fn eval(&self, sc: &mut Scope) -> Result<Value, ActionError> {
        //println!("eval {}",self.print());
        use Expr::*;
        Ok(match self {
            Bool(b) => Value::Bool(*b),
            Num(n) => Value::Num(*n),
            Str(s) => Value::Str(s.clone()),

            Bracket(a) => a.eval(sc)?,
            Neg(a) => a.eval(sc)?.try_neg()?,
            Oper(Op::Dot, _, _) | Ident(_) | DotStart(_) | Rooted(_) => {
                let proto = self.eval_path(sc)?;
                let v = sc.get(&proto).ok_or(ActionError::new("Nothing at path"))?;
                v.clone_weak().to_strong(sc.gm_mut())
            }
            Oper(o, a, b) => o.eval(a, b, sc)?,
            List(ref l) => {
                let mut res = Vec::new();
                for e in l {
                    let v = e.eval(sc)?;
                    res.push(sc.push_mem(v));
                }
                Value::List(res)
            }
            Map(ref l) => {
                let mut res = BTreeMap::new();
                for e in l {
                    let v = e.v.eval(sc)?;
                    res.insert(ProtoNode::str(&e.k), sc.push_mem(v));
                }
                Value::Map(res)
            }
            If(bex, l_block, r_block) => {
                let v = bex.eval(sc)?;
                let op = match v {
                    Value::Bool(true) => l_block,
                    Value::Bool(false) => r_block,
                    _ => return Err(ActionError::new("If only works on bool expr")),
                };
                sc.on_wrap(|sc2| sc2.do_actions(op)).map(|v| v.1)?
            }
            _ => unimplemented!(),
        })
    }

    pub fn eval_action(&self, sc: &mut Scope) -> AcResult {
        match self {
            Self::If(bex, l_block, r_block) => {
                let v = bex.eval(sc)?;
                let op = match v {
                    Value::Bool(true) => l_block,
                    Value::Bool(false) => r_block,
                    _ => return Err(ActionError::new("If only works on bool expr")),
                };
                sc.on_wrap(|sc2| sc2.do_actions(op))
            }
            _ => self.eval(sc).map(|v| (AcReturn::No, v)),
        }
    }
    pub fn add_left(self, lf: Expr, op: Op) -> Self {
        match self {
            Expr::Oper(sop, sa, sb) => {
                if sop.rank() >= op.rank() {
                    Expr::Oper(sop, Box::new(sa.add_left(lf, op)), sb)
                } else {
                    Expr::Oper(op, Box::new(lf), Box::new(Expr::Oper(sop, sa, sb)))
                }
            }
            e => Expr::Oper(op, Box::new(lf), Box::new(e)),
        }
    }

    pub fn print(&self) -> String {
        use Expr::*;
        match self {
            Num(n) => n.to_string(),
            Str(s) => format!("\"{}\"", s),
            Oper(o, a, b) => format!("{}{}{})", a.print(), o.to_str(), b.print()),
            Neg(a) => format!("-{}", a.print()),
            Bracket(b) => format!("({})", b.print()),
            e => format!("{:?}", e),
        }
    }
}
#[cfg(test)]
mod test_expr {
    use super::*;
    #[test]
    fn test_expr_results() {
        let scope = Scope::new();
        let r: Expr = "(5 + 2)".parse().unwrap();
        assert_eq!(r.eval(&scope), Ok(Value::Num(7)));

        let r: Expr = "(5 +2 *2)".parse().unwrap();
        assert_eq!(r.eval(&scope), Ok(Value::Num(9)));

        /*let r: Expr = "((3+4) * (10-1))".parse().unwrap();
        assert_eq!(r.eval(&scope), Ok(Value::Num(63)));

        let r: Expr = "(3 +5 +4 +7 +2)".parse().unwrap();
        assert_eq!(r.eval(&scope), Ok(Value::Num(21)));
        */
    }
}
