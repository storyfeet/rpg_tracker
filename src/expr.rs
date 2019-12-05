use crate::dndata::DnData;
use crate::action::Action;
use crate::error::{ActionError, LineError};
use crate::prev_iter::LineCounter;
use crate::proto::Proto;
use crate::token::{TokPrev, Token};
use crate::value::Value;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;
use crate::prev_iter::Backer;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Num(i32),
    Proto(Proto),
    Func(Proto,Vec<Value>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Op(Token),
}

impl Add for Expr {
    type Output = Expr;
    fn add(self, rhs: Expr) -> Self::Output {
        use Expr::*;
        match self {
            Num(a) => match rhs {
                Num(b) => Num(a + b),
                e => Add(Box::new(Num(a)), Box::new(e)),
            },
            a => Add(Box::new(a), Box::new(rhs)),
        }
    }
}

impl Sub for Expr {
    type Output = Expr;
    fn sub(self, rhs: Expr) -> Self::Output {
        use Expr::*;
        match self {
            Num(a) => match rhs {
                Num(b) => Num(a - b),
                e => Sub(Box::new(Num(a)), Box::new(e)),
            },
            a => Sub(Box::new(a), Box::new(rhs)),
        }
    }
}
impl Mul for Expr {
    type Output = Expr;
    fn mul(self, rhs: Expr) -> Self::Output {
        use Expr::*;
        match self {
            Num(a) => match rhs {
                Num(b) => Num(a * b),
                e => Mul(Box::new(Num(a)), Box::new(e)),
            },
            a => Mul(Box::new(a), Box::new(rhs)),
        }
    }
}

impl Div for Expr {
    type Output = Expr;
    fn div(self, rhs: Expr) -> Self::Output {
        use Expr::*;
        match self {
            Num(a) => match rhs {
                Num(0) => Num(0),
                Num(b) => Num(a / b),
                e => Mul(Box::new(Num(a)), Box::new(e)),
            },
            a => Mul(Box::new(a), Box::new(rhs)),
        }
    }
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
    pub fn eval(&self, root: &mut DnData) -> Result<Value, ActionError> {
        use Expr::*;
        Ok(match self {
            Num(n) => Value::num(*n),
            Proto(p) => root.resolve(Value::Proto(p.with_deref(1)))?,
            Add(a, b) => a.eval(root)?.try_add(b.eval(root)?)?,
            Sub(a, b) => a.eval(root)?.try_sub(b.eval(root)?)?,
            Mul(a, b) => a.eval(root)?.try_mul(b.eval(root)?)?,
            Div(a, b) => a.eval(root)?.try_div(b.eval(root)?)?,
            Neg(a) => a.eval(root)?.try_neg()?,
            Func(nm,params) => root.do_action(Action::CallFunc(nm.clone(),params.to_vec()))?.ok_or(ActionError::new("func in expression returns no value"))?,
            _ => Value::num(0),
        })
    }

    pub fn neg(self) -> Self {
        match self {
            Expr::Num(n) => Expr::Num(-n),
            Expr::Neg(ex) => *ex,
            ex => Expr::Neg(Box::new(ex)),
        }
    }

    pub fn print(&self) -> String {
        use Expr::*;
        match self {
            Num(n) => n.to_string(),
            Proto(p) => format!("{}", p),
            Add(a, b) => format!("({}+{})", a.print(), b.print()),
            Sub(a, b) => format!("({}-{})", a.print(), b.print()),
            Mul(a, b) => format!("({}*{})", a.print(), b.print()),
            Div(a, b) => format!("({}/{})", a.print(), b.print()),
            Neg(a) => format!("-{}", a.print()),
            e => format!("{:?}", e),
        }
    }

    pub fn from_tokens(it: &mut TokPrev) -> Result<Expr, LineError> {
        let mut parts = Vec::new();
        while let Some(t) = it.next() {
            match t {
                Token::Dollar => {
                    let pt = Proto::from_tokens(it);
                    match it.next() {
                        Some(Token::BOpen)=>{
                            let mut params = Vec::new();
                            while let Some(tk) = it.next(){
                                match tk{
                                    Token::BClose=>{
                                        parts.push(Expr::Func(pt,params));
                                        break;
                                    }
                                    Token::Comma=>{}
                                    _=>{
                                        it.back();
                                        params.push(Value::from_tokens(it)?);
                                    }
                                }
                            }
                        }
                        _ =>{
                            it.back();
                            parts.push(Expr::Proto(pt));
                        }
                    }
                }
                Token::Break | Token::BClose => break,
                Token::BOpen => parts.push(Self::from_tokens(it)?),
                Token::Add | Token::Sub | Token::Mul | Token::Div => parts.push(Expr::Op(t)),
                Token::Num(n) => parts.push(Expr::Num(n)),
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
