use crate::token::Token;
use crate::expr::Expr;
use crate::error::LineError;
use crate::token::TokPrev;
use crate::prev_iter::{Backer,LineCounter};
use crate::value::Value;

#[derive(PartialEq, Clone, Debug)]
pub struct ProtoX{
    pub d:i32,
    pub dot:bool,
    pub var:bool,
    pub exs:Vec<Expr>,
}

impl ProtoX{
    pub fn from_tokens(t: &mut TokPrev) -> Result<Self, LineError> {
        let mut res = ProtoX{
            d:0,
            dot:false,
            var:false,
            exs:Vec::new(),
        };


        match t.next() {
            Some(Token::Var)=>res.var = true,
            Some(Token::Dot)=>res.dot = true,
            _=>t.back()
        }
        while let Some(Token::Dollar) = t.next(){
            res.d +=1;
        }
        t.back();

        while let Some(v) = t.next() {
            match v {
                Token::Dot => return Err(t.err("Double Dot")),
                Token::Qoth(s) | Token::Ident(s) => res.exs.push(Expr::Val(Value::Str(s))),
                Token::Num(n) => res.exs.push(Expr::num(n)),
                _ => {
                    t.back();
                    res.exs.push(Expr::from_tokens(t)?);
                }
            }

            match t.next(){
                Some(Token::Dot)=>{},
                _=>{
                    t.back();
                    return Ok(res);
                }

            }
        }
        Ok(res)
    }
}
