use crate::error::{ActionError, LineError};
use crate::prev_iter::Backer;
use crate::prev_iter::LineCounter;
use crate::proto::Proto;
use crate::proto_ex::ProtoX;
use crate::scope::Scope;
use crate::token::{TokPrev, Token};
use crate::value::Value;
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Val(Value),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    Greater(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Op(Token),
    List(Vec<Expr>),
    Map(BTreeMap<String, Expr>),
    ProtoEx(ProtoX), //Also covers call func
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
            Add(a, b) => a.eval(scope)?.try_add(b.eval(scope)?)?,
            Sub(a, b) => a.eval(scope)?.try_sub(b.eval(scope)?)?,
            Mul(a, b) => a.eval(scope)?.try_mul(b.eval(scope)?)?,
            Div(a, b) => a.eval(scope)?.try_div(b.eval(scope)?)?,
            Neg(a) => a.eval(scope)?.try_neg()?,
            Greater(a, b) => Value::Bool(a.eval(scope)? > b.eval(scope)?),
            Less(a, b) => Value::Bool(a.eval(scope)? < b.eval(scope)?),
            Equal(a, b) => Value::Bool(a.eval(scope)? == b.eval(scope)?),
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
            Op(_)=>return Err(ActionError::new("Operator not a complete expression")),
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
            Token::BracketO => {} // pass on to expr sum
            Token::SquareO => {
                let mut parts = Vec::new();
                while let Some(t) = it.next() {
                    match t {
                        Token::Break | Token::Comma => {}
                        Token::SquareC => return Ok(Expr::List(parts)),
                        _ => {
                            it.back();
                            parts.push(Expr::from_tokens(it)?);
                        }
                    }
                }
                return Err(it.eof());
            }
            Token::SquigleO => {
                let mut map = BTreeMap::new();
                while let Some(t) = it.next() {
                    match t {
                        Token::Break | Token::Comma => {}
                        Token::SquigleC => return Ok(Expr::Map(map)),
                        Token::Ident(s) | Token::Qoth(s) => {
                            it.next(); //colon
                            let ex = Expr::from_tokens(it)?;
                            map.insert(s, ex);
                        }
                        e => return Err(it.ux(e, "at treeexpr")),
                    }
                }
            }
            Token::Sub => return Ok(Expr::neg(Expr::from_tokens(it)?)),
            Token::Ident(_) | Token::Dot| Token::Dollar => {
                it.back();
                return Ok(Expr::ProtoEx(ProtoX::from_tokens(it)?));
            }
            _ => {
                it.back();
                return Ok(Expr::Val(Value::from_tokens(it)?));
            }
        }
        //only get here if Bracket Open '('
        let mut parts = Vec::new();
        let mut is_first = true;
        while let Some(t) = it.next() {
            match t {
                Token::Break | Token::BracketC => break,
                Token::BracketO => parts.push(Self::from_tokens(it)?),
                Token::Add
                | Token::Mul
                | Token::Div
                | Token::Greater
                | Token::Less
                | Token::Equals => parts.push(Expr::Op(t)),
                Token::Sub => {
                    if is_first {
                        parts.push(Expr::Neg(Box::new(Expr::from_tokens(it)?)));
                    } else {
                        parts.push(Expr::Op(Token::Sub));
                    }
                }
                Token::Dollar | Token::Ident(_) | Token::Var => {
                    it.back();
                    let p = ProtoX::from_tokens(it)?;
                    parts.push(Expr::ProtoEx(p));
                }
                _ => {
                    it.back();
                    parts.push(Expr::from_tokens(it)?);
                }
            }
            is_first = false;
        }

        let p = parts;
        let p = Self::split_op(p, Token::Mul, |a, b| Expr::Mul(Box::new(a), Box::new(b)))?;
        let p = Self::split_op(p, Token::Div, |a, b| Expr::Div(Box::new(a), Box::new(b)))?;
        let p = Self::split_op(p, Token::Sub, |a, b| Expr::Sub(Box::new(a), Box::new(b)))?;
        let p = Self::split_op(p, Token::Add, |a, b| Expr::Add(Box::new(a), Box::new(b)))?;
        let p = Self::split_op(p, Token::Greater, |a, b| {
            Expr::Greater(Box::new(a), Box::new(b))
        })?;
        let p = Self::split_op(p, Token::Less, |a, b| Expr::Less(Box::new(a), Box::new(b)))?;
        let p = Self::split_op(p, Token::Equals, |a, b| {
            Expr::Equal(Box::new(a), Box::new(b))
        })?;

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
        let scope = Scope::new();
        let r: Expr = "(5 + 2)".parse().unwrap();
        assert_eq!(r.eval(&scope),Ok(Value::Num( 7)));

        let r: Expr = "(5 +2 *2)".parse().unwrap();
        assert_eq!(r.eval(&scope), Ok(Value::Num(9)));

        /*let r: Expr = "((3+4) * (10-1))".parse().unwrap();
        assert_eq!(r.eval(&scope), Ok(Value::Num(63)));

        let r: Expr = "(3 +5 +4 +7 +2)".parse().unwrap();
        assert_eq!(r.eval(&scope), Ok(Value::Num(21)));
        */
    }
}
