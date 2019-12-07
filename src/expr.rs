use crate::error::{ActionError, LineError};
use crate::prev_iter::Backer;
use crate::prev_iter::LineCounter;
use crate::proto::Proto;
use crate::scope::Scope;
use crate::token::{TokPrev, Token};
use crate::value::Value;
use std::str::FromStr;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Val(Value),
    Func(Proto, Vec<Value>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    LThan(Box<Expr>, Box<Expr>),
    GThan(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Op(Token),
}

impl FromStr for Expr {
    type Err = LineError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut t = TokPrev::new(s);
        let e = Self::from_tokens(&mut t)?;
        Ok(e)
    }
}

impl Expr {
    pub fn num(n: i32) -> Self {
        Expr::Val(Value::Num(n))
    }
    pub fn neg(e:Expr)->Self{
        Expr::Neg(Box::new(e))
    }

    pub fn eval(&self, scope: &Scope) -> Result<Value, ActionError> {
        //println!("eval {}",self.print());
        use Expr::*;
        Ok(match self {
            Val(n) => scope.resolve(n)?,
            Add(a, b) => a.eval(scope)?.try_add(b.eval(scope)?)?,
            Sub(a, b) => a.eval(scope)?.try_sub(b.eval(scope)?)?,
            Mul(a, b) => a.eval(scope)?.try_mul(b.eval(scope)?)?,
            Div(a, b) => a.eval(scope)?.try_div(b.eval(scope)?)?,
            Neg(a) => a.eval(scope)?.try_neg()?,
            //LThan(a, b) => Value::num((a.eval(scope)? < b.eval(scope)?) as i32),
            Func(nm, params) => scope
                .call_func_const(nm.clone(), params)?
                .ok_or(ActionError::new("func in expression returns no value"))?,
            _ => Value::Num(0),
        })
    }

    pub fn print(&self) -> String {
        use Expr::*;
        match self {
            Val(v) => v.print(0),
            Add(a, b) => format!("({}+{})", a.print(), b.print()),
            Sub(a, b) => format!("({}-{})", a.print(), b.print()),
            Mul(a, b) => format!("({}*{})", a.print(), b.print()),
            Div(a, b) => format!("({}/{})", a.print(), b.print()),
            Neg(a) => format!("-{}", a.print()),
            e => format!("{:?}", e),
        }
    }

    pub fn from_tokens(it: &mut TokPrev) -> Result<Expr, LineError> {
        match it.next().ok_or(it.eof())? {
            Token::BOpen=>{},
            Token::Sub=>return Ok(Expr::neg(Expr::from_tokens(it)?)),
            _ => {
                it.back();
                return Ok(Expr::Val(Value::from_tokens(it)?));
            }
        }
        let mut parts = Vec::new();
        while let Some(t) = it.next() {
            match t {
                Token::Dollar => {
                    let pt = Proto::from_tokens(it);
                    }
                }
                Token::Break | Token::BClose => break,
                Token::BOpen => parts.push(Self::from_tokens(it)?),
                Token::Add
                | Token::Sub
                | Token::Mul
                | Token::Div
                | Token::GThan
                | Token::LThan
                | Token::Amp
                | Token::Or => parts.push(Expr::Op(t)),
                Token::Num(n) => parts.push(Expr::Val(Value::Num(n))),
                _ => return Err(it.err("Unexptected token in expression")),
            }
        }

        let p = Self::split_op(parts, Token::Mul, |a, b| {
            Expr::Mul(Box::new(a), Box::new(b))
        })?;
        let p = Self::split_op(p, Token::Div, |a, b| Expr::Div(Box::new(a), Box::new(b)))?;
        let p = Self::split_op(p, Token::Sub, |a, b| Expr::Sub(Box::new(a), Box::new(b)))?;
        let p = Self::split_op(p, Token::Add, |a, b| Expr::Add(Box::new(a), Box::new(b)))?;

        Ok(p[0].clone())
    }

    pub fn split_op<IT, F>(i: IT, t: Token, f: F) -> Result<Vec<Expr>, LineError>
    where
        IT: IntoIterator<Item = Expr> + std::fmt::Debug,
        F: Fn(Expr, Expr) -> Expr,
    {
        //        println!("{:?}",i);
        // mul and div
        let mut a = None;
        let mut res = Vec::new();
        let mut pit = i.into_iter();
        while let Some(p) = pit.next() {
            if p == Expr::Op(t.clone()) {
                a = Some(f(
                    a.take().ok_or(LineError::new("nothing berfore the *", 0))?,
                    pit.next().ok_or(LineError::new("Nothing after the *", 0))?,
                ));
            } else {
                if let Some(prev) = a {
                    res.push(prev)
                }
                a = Some(p);
            }
        }
        if let Some(av) = a {
            res.push(av)
        }
        Ok(res)
    }
}

#[cfg(test)]
mod test_expr {
    use super::*;
    #[test]
    fn test_expr_results() {
        let r: Expr = "5 +2".parse().unwrap();
        assert_eq!(r.eval(), 7);

        let r: Expr = "5 +2 *2".parse().unwrap();
        assert_eq!(r.eval(), 9);

        let r: Expr = "(3+4)*(10-1)".parse().unwrap();
        assert_eq!(r.eval(), 63);

        let r: Expr = "3 +5 +4 +7 +2".parse().unwrap();
        assert_eq!(r.eval(), 21);
    }
}
