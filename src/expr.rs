use crate::action::Action;
use crate::error::{ActionError, LineError};
//use crate::prev_iter::Backer;
//use crate::prev_iter::LineCounter;
use crate::proto::Proto;
use crate::scope::Scope;
use crate::value::Value;
use gobble::err::ECode;
use std::collections::BTreeMap;
use std::str::FromStr;

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
            Dot => 11,
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
}

#[derive(PartialEq, Debug, Clone)]
pub struct MapItem {
    k: String,
    v: Expr,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Bool(bool),
    Num(i64),
    Str(String),
    Oper(Op, Box<Expr>, Box<Expr>),
    Bracket(Box<Expr>),
    Neg(Box<Expr>),
    DotStart(Box<Expr>),
    List(Vec<Expr>),
    Map(Vec<MapItem>),
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
            Num(n) => Proto::num(n),
            Str(s) => Proto::one(s),
            Oper(Op::Dot, a, b) => a.eval_path(sc)?.extend_new(b.eval_path(sc)?.pp()),
            ot => match ot.eval(sc)?{
                Value::Num(n)=>Proto::num(n),
                Value::Str(s)=>Proto::one(s),
                ov => {
                    ov.gen_drop(sc.gm_mut());
                    return Err(ActionError::new("Could not treat as proto")),
                }

            }
        })
    }

    pub fn eval(&self, scope: &mut Scope) -> Result<Value, ActionError> {
        //println!("eval {}",self.print());
        use Expr::*;
        Ok(match self {
            Val(n) => n.clone(scope),
            Bracket(a) => a.eval(scope)?,
            Neg(a) => a.eval(scope)?.try_neg()?,
            Oper(Op::Add, a, b) => a.eval(scope)?.try_add(b.eval(scope)?)?,
            Oper(Op::Sub, a, b) => a.eval(scope)?.try_sub(b.eval(scope)?)?,
            Oper(Op::Mul, a, b) => a.eval(scope)?.try_mul(b.eval(scope)?)?,
            Oper(Op::Div, a, b) => a.eval(scope)?.try_div(b.eval(scope)?)?,
            Oper(Op::Greater, a, b) => Value::Bool(a.eval(scope)? > b.eval(scope)?),
            Oper(Op::Less, a, b) => Value::Bool(a.eval(scope)? < b.eval(scope)?),
            Oper(Op::Equal, a, b) => Value::Bool(a.eval(scope)? == b.eval(scope)?),
            Oper(Op::Dot, a, b) => {
                let proto = self.eval_path(scope)?;
                Value::Ref(proto)
            }
            List(ref l) => Value::List(l.iter().map(|e| scope.push_mem(e.eval(scope))).collect()),
            Map(ref l) => {
                let mut t = BTreeMap::new();
                eval_map_expr(l, &mut t, scope)?;
                Value::Map(t)
            }
            Oper(Op::Dot, a, b) => Value::Proto(Proto::join(a.eval(scope)?, b.eval(scope)?)?),
        })
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
            Val(v) => v.print(0),
            Oper(o, a, b) => format!("{}{}{})", a.print(), o.to_str(), b.print()),
            Neg(a) => format!("-{}", a.print()),
            Bracket(b) => format!("({})", b.print()),
            e => format!("{:?}", e),
        }
    }

    pub fn from_tokens(_: &mut TokPrev) -> Result<Expr, LineError> {
        unimplemented!();
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
