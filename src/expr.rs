use crate::error::{ActionError, LineError};
//use crate::prev_iter::Backer;
//use crate::prev_iter::LineCounter;
use crate::proto::Proto;
use crate::proto_ex::ProtoX;
use crate::scope::Scope;
use crate::token::TokPrev;
use crate::value::Value;
use std::collections::BTreeMap;
use std::str::FromStr;
use crate::nomp::r_expr;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Op {
    Add,
    Sub,
    Div,
    Mul,
    Greater,
    Less,
    Equal,
}

impl Op {
    pub fn from_char(c: char) -> Self {
        use Op::*;
        match c {
            '+' => Add,
            '-' => Sub,
            '/' => Div,
            '*' => Mul,
            '>' => Greater,
            '<' => Less,
            _ => Equal,
        }
    }
    pub fn rank(&self) -> i32 {
        use Op::*;
        match self {
            Add => 10,
            Sub => 9,
            Mul => 8,
            Div => 7,
            Greater => 6,
            Less => 5,
            Equal => 4,
        }
    }

    pub fn char(&self)->char{
        use Op::*;
        match self {
            Add => '+',
            Sub => '-',
            Mul => '*',
            Div => '/',
            Greater => '>',
            Less => '<',
            Equal => '=',
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Val(Value),
    Oper(Op, Box<Expr>, Box<Expr>),
    Bracket(Box<Expr>),
    Neg(Box<Expr>),
    List(Vec<Expr>),
    Map(BTreeMap<String, Expr>),
    ProtoEx(ProtoX), //Also covers call func
}

impl FromStr for Expr {
    type Err = LineError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        r_expr(s).map(|(_,a)|a).map_err(|_|LineError::new("could not make Expr from str",0))

    }
}

impl Expr {
    pub fn num(n: i32) -> Self {
        Expr::Val(Value::Num(n))
    }
    pub fn neg(e: Expr) -> Self {
        Expr::Neg(Box::new(e))
    }
    pub fn proto(p: Proto) -> Self {
        Expr::Val(Value::Proto(p))
    }

    pub fn eval(&self, scope: &Scope) -> Result<Value, ActionError> {
        //println!("eval {}",self.print());
        use Expr::*;
        Ok(match self {
            Val(n) => n.clone(),
            Oper(Op::Add, a, b) => a.eval(scope)?.try_add(b.eval(scope)?)?,
            Oper(Op::Sub, a, b) => a.eval(scope)?.try_sub(b.eval(scope)?)?,
            Oper(Op::Mul, a, b) => a.eval(scope)?.try_mul(b.eval(scope)?)?,
            Oper(Op::Div, a, b) => a.eval(scope)?.try_div(b.eval(scope)?)?,
            Bracket(a) => a.eval(scope)?,
            Neg(a) => a.eval(scope)?.try_neg()?,
            Oper(Op::Greater, a, b) => Value::Bool(a.eval(scope)? > b.eval(scope)?),
            Oper(Op::Less, a, b) => Value::Bool(a.eval(scope)? < b.eval(scope)?),
            Oper(Op::Equal, a, b) => Value::Bool(a.eval(scope)? == b.eval(scope)?),
            List(l) => {
                let mut vl = Vec::new();
                for e in l {
                    vl.push(e.eval(scope)?);
                }
                Value::List(vl)
            }
            Map(m) => {
                let mut t = Value::tree();
                for (k, v) in m.iter() {
                    t.set_at_path(Proto::one(k).pp(), v.eval(scope)?);
                }
                t
            }
            ProtoEx(p) => p.eval_expr(scope)?,
        })
    }

    pub fn add_left(self, lf: Expr, op: Op) -> Self {
        match self {
            Expr::Oper(sop, sa, sb) => {
                if sop.rank() >= op.rank() {
                    Expr::Oper(sop, Box::new(sa.add_left(lf,op)), sb)
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
            Oper(o, a, b) => format!("{}{}{})", a.print(),o.char(), b.print()),
            Neg(a) => format!("-{}", a.print()),
            Bracket(b) => format!("({})",b.print()),
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
