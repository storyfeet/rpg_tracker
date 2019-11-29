use crate::error::LineError;
use crate::token::{Token, Tokenizer};
use std::ops::{Add, Sub};
use std::str::FromStr;
use crate::prev_iter::LineCounter;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Num(i32),
    Ident(String),
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
impl FromStr for Expr {
    type Err = LineError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut t = Tokenizer::new(s);
        let e = Self::from_tokens(&mut t)?;
        Ok(e)
    }
}

impl Expr {
    fn eval(&self) -> i32 {
        use Expr::*;
        match self {
            Num(n) => *n,
            Ident(_) => 0, //TODO
            Add(a, b) => a.eval() + b.eval(),
            Sub(a, b) => a.eval() - b.eval(),
            Mul(a, b) => a.eval() * b.eval(),
            Div(a, b) => {
                let bb = b.eval();
                if bb == 0 {
                    0
                } else {
                    a.eval() / bb
                }
            }
            Neg(a) => -a.eval(),
            _ => 0,
        }
    }

    fn push_ident(&mut self, i: &str) -> Result<(), LineError> {
        if let Expr::Ident(ref mut s) = self {
            s.push('.');
            s.push_str(i);
            return Ok(());
        }
        Err(LineError::new("Cannot add dotted ident to non ident", 0))
    }

    pub fn neg(self) -> Self {
        match self {
            Expr::Num(n) => Expr::Num(-n),
            Expr::Neg(ex) => *ex,
            ex => Expr::Neg(Box::new(ex)),
        }
    }

    pub fn from_tokens<T:Iterator<Item=Token>+LineCounter>(it: &mut T) -> Result<Expr, LineError> {
        let mut parts = Vec::new();
        while let Some(t) = it.next() {
            match t {
                Token::Ident(s) => parts.push(Expr::Ident(s)),
                Token::Dot => match it.next() {
                    Some(Token::Ident(s)) => {
                        let plen = parts.len();
                        parts
                            .get_mut(plen - 1)
                            .ok_or(it.err("Cannot start with dot"))?
                            .push_ident(&s)
                            .map_err(|p| p.set_line(it.line()))?;
                    }
                    None | Some(_) => {
                        return Err(it.err("Expected idend after dot"))
                    }
                },
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
                    a.take()
                        .ok_or(LineError::new("nothing berfore the *", 0))?,
                    pit.next()
                        .ok_or(LineError::new("Nothing after the *", 0))?,
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
