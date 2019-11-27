use crate::expr::Expr;
use std::ops::{Add, Sub};
use crate::token::{Token,Tokenizer};
use crate::error::ParseError;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Ex(Expr),
    Str(String),
    List(Vec<String>),
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Value) -> Value {
        use Value::*;
        match self {
            Null => rhs,
            Ex(mut a) => match rhs {
                Null=>Ex(a),
                Ex(b) => Ex(a + b),
                Str(b) => Ex(a + Expr::Ident(b)),
                List(bb) => {
                    for b in bb {
                        a = a + Expr::Ident(b);
                    }
                    Ex(a)
                }
            },
            Str(mut a) => match rhs {
                Null => Str(a),
                Ex(b) => {
                    a.push_str(&format!("{:?}", b));
                    Str(a)
                }
                Str(b) => {
                    a.push_str(&b.to_string());
                    Str(a)
                }
                List(bb) => {
                    for b in bb {
                        a.push_str(&b)
                    }
                    Str(a)
                }
            },
            List(mut a) => match rhs {
                Null => List(a),
                Ex(b) => {
                    a.push(format!("{:?}", b));
                    List(a)
                }
                Str(b) => {
                    a.push(b);
                    List(a)
                }
                List(b) => {
                    a.extend(b);
                    List(a)
                }
            },
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Value) -> Value {
        use Value::*;
        match self {
            Null => Null,
            Ex(mut a) => match rhs {
                Null => Ex(a),
                Ex(b) => Ex(a - b),
                Str(b) => Ex(a - Expr::Ident(b)),
                List(bb) => {
                    for b in bb {
                        a = a - Expr::Ident(b);
                    }
                    Ex(a)
                }
            },
            Str(mut a) => match rhs {
                Null=>
                Ex(_) => Null,
                Str(b) => {
                    a.push_str(&b.to_string());
                    Str(a)
                }
                List(bb) => {
                    for b in bb {
                        a.push_str(&b)
                    }
                    Str(a)
                }
            },
            List(mut a) => match rhs {
                Ex(b) => {
                    a.push(format!("{:?}", b));
                    List(a)
                }
                Str(b) => {
                    a.push(b);
                    List(a)
                }
                List(b) => {
                    a.extend(b);
                    List(a)
                }
            },
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
    pub fn from_tokens(t:&mut Tokenizer)->Result<Self,ParseError>{
        match t.next(){
            None=>Err(ParseError::new("UX-EOF",t.line_no)),
            Some(Token::BOpen)=>Expr::from_tokens(t).map(|v|Value::Ex(v)),
            Some(Token::BOpen)=>{
                let mut rlist = Vec::new();
                while let Some(v) = t.next(){
                    match v {
                        Token::Qoth(s)|Token::Ident(s)=>rlist.push(s),
                        Token::Num(n)=>rlist.push(n.to_string()),
                        Token::SBClose => return Ok(Value::List(rlist)),
                        e=>return Err(ParseError::new(&format!("UX - {:?}",e),t.line_no)),
                    }
                }
                Err(ParseError::new("UX-EOF",t.line_no))

            },
            Some(Token::Ident(s))|Some(Token::Qoth(s))=> Ok(Value::Str(s)),
            Some(Token::Num(n))=>Ok(Value::Ex(Expr::Num(n))),
            v=>Err(ParseError::new(&format!("UX - {:?}",v),t.line_no)),
        }
    }
}
