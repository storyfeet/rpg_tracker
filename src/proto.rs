use crate::error::LineError;
use crate::prev_iter::LineCounter;
use crate::expr::Expr;
use crate::prev_iter::Backer;
use crate::token::{TokPrev, Token};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoNode {
    Num(i32),
    Expr(Expr),
    Str(String),
}

impl ProtoNode {
    pub fn str(s: &str) -> Self {
        ProtoNode::Str(s.to_string())
    }

    pub fn as_string(&self)->String{
        match self{
            ProtoNode::Num(n)=>n.to_string(),
            ProtoNode::Expr(e)=>format!("{}",e.print()),
            ProtoNode::Str(s)=>s.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proto {
    pub dotted: bool,
    pub derefs: i32,
    pub var:bool,
    v: Vec<ProtoNode>,
}

impl Display for Proto {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for _ in 0..self.derefs {
            write!(f, "$")?;
        }
        if self.dotted {
            write!(f, ".")?;
        }
        for (i, node) in self.v.iter().enumerate() {
            if i != 0 {
                write!(f, ".")?;
            }
            match node {
                ProtoNode::Num(n) => write!(f, "{}", n)?,
                ProtoNode::Expr(e) => write!(f, "{:?}", e)?,
                ProtoNode::Str(s) => write!(f, "{}", s.replace(".", "\\."))?,
            }
        }
        Ok(())
    }
}

impl Proto {
    pub fn empty(dotted: bool) -> Self {
        Proto {
            dotted,
            derefs: 0,
            var:false,
            v: Vec::new(),
        }
    }
    pub fn one(s: &str, dotted: bool) -> Self {
        Proto {
            dotted,
            var:false,
            v: vec![ProtoNode::str(s)],
            derefs: 0,
        }
    }

    pub fn var(s: &str) -> Self {
        Proto {
            dotted: false,
            var:true,
            v: vec![ProtoNode::str("var"), ProtoNode::str(s)],
            derefs: 0,
        }
    }
    pub fn new(s: &str) -> Result<Self,LineError> {
        let mut t = TokPrev::new(s);
        Self::from_tokens(&mut t)
    }

    pub fn as_func_name(&self)->&str{
        match self.v.get(0){
            Some(ProtoNode::Str(s))=>s,
            _=>"",
        }
    }

    pub fn join_on_dot(&self, p: Self) -> Self {
        if !p.dotted {
            return p;
        }
        self.extend_new(p.pp())
    }

    pub fn parent(&self) -> Self {
        let mut res = self.clone();
        if res.v.len() > 0 {
            res.v.pop();
        }
        res
    }

    pub fn from_str(s: &str) -> Result<Self,LineError> {
        let mut tp = TokPrev::new(s);
        Self::from_tokens(&mut tp)
    }

    pub fn from_tokens(t: &mut TokPrev) -> Result<Self, LineError> {
        let mut res = Proto::empty(false);

        match t.next() {
            Some(Token::Var)=>res.var = true,
            Some(Token::Dot)=>res.dotted = true,
            _=>t.back()
        }
        while let Some(Token::Dollar) = t.next(){
            res.derefs +=1;
        }
        t.back();

        while let Some(v) = t.next() {
            match v {
                Token::Dot => return Err(t.err("Double Dot")),
                Token::Qoth(s) | Token::Ident(s) => res.v.push(ProtoNode::str(&s)),
                Token::Num(n) => res.v.push(ProtoNode::Num(n)),
                _ => {
                    t.back();
                    res.v.push(ProtoNode::Expr(Expr::from_tokens(t)?));
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

    pub fn push(&mut self, s: &str) {
        self.v.push(ProtoNode::str(s))
    }

    pub fn pp<'a>(&'a self) -> ProtoP<'a> {
        ProtoP {
            p: &self,
            pos: 0,
            stop: self.v.len(),
        }
    }

    pub fn extend(&mut self, pp: ProtoP) {
        self.v.extend(pp.map(|s| s.clone()))
    }

    pub fn extend_new(&self, pp: ProtoP) -> Self {
        let mut res = self.clone();
        res.extend(pp);
        res
    }

    pub fn with_deref(&self, n: i32) -> Proto {
        let mut res = self.clone();
        res.derefs += n;
        res
    }

    pub fn with_set_deref(&self, n: i32) -> Proto {
        let mut res = self.clone();
        res.derefs = n;
        res
    }
}

#[derive(Clone, Debug)]
pub struct ProtoP<'a> {
    p:&'a Proto,
    pos: usize,
    stop: usize,
}

impl<'a> Iterator for ProtoP<'a> {
    type Item = &'a ProtoNode;
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.pos;
        self.pos += 1;
        if n >= self.stop {
            return None;
        }
        Some(&self.p.v[n])
    }
}

impl<'a> ProtoP<'a> {
    pub fn remaining(&self) -> usize {
        if self.pos > self.p.v.len() {
            return 0;
        }
        self.p.v.len() - self.pos
    }

    pub fn var(&self)->bool{
        self.p.var
    }
    pub fn parent(&self) -> Self {
        let mut res = self.clone();
        if res.stop >= 1 {
            res.stop -= 1;
        }
        res
    }
}
