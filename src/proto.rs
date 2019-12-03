use crate::prev_iter::{Backer, LineCounter, Prev};
use crate::token::{Token, Tokenizer};
#[derive(Debug, Clone, PartialEq)]
pub struct Proto {
    pub dots: i32,
    pub derefs: i32,
    v: Vec<String>,
}

impl Proto {
    pub fn empty(dots: i32) -> Self {
        Proto {
            dots,
            derefs: 0,
            v: Vec::new(),
        }
    }
    pub fn one(s: &str, dots: i32) -> Self {
        let mut v = Vec::new();
        v.push(s.to_string());
        Proto { dots, v, derefs: 0 }
    }
    pub fn new(s: &str) -> Self {
        let mut t = Prev::new(Tokenizer::new(s));
        Self::from_tokens(&mut t)
    }

    pub fn from_tokens<T: Iterator<Item = Token> + Backer + LineCounter>(t: &mut T) -> Self {
        let mut res = Proto::empty(0);
        while let Some(v) = t.next() {
            match v {
                Token::Dot => {
                    if res.v.len() == 0 {
                        res.dots += 1
                    }
                }
                Token::Mul => {
                    if res.v.len() == 0 {
                        res.derefs += 1
                    }
                }
                Token::Qoth(s) | Token::Ident(s) => res.push(&s),
                _ => {
                    t.back();
                    return res;
                }
            }
        }
        res
    }

    pub fn push(&mut self, s: &str) {
        self.v.push(s.to_string())
    }

    pub fn pp<'a>(&'a self) -> ProtoP<'a> {
        ProtoP { v: &self.v, pos: 0 }
    }

    pub fn extend(&mut self, pp: ProtoP) {
        self.v.extend(pp.map(|s| s.to_string()))
    }

    pub fn extend_new(&self, pp: ProtoP) -> Self {
        let mut res = self.clone();
        res.extend(pp);
        res
    }

    pub fn with_deref(&self,n:i32)->Proto{
        let mut res = self.clone();
        res.derefs +=n;
        res
    }
}

#[derive(Clone)]
pub struct ProtoP<'a> {
    v: &'a Vec<String>,
    pos: usize,
}

impl<'a> Iterator for ProtoP<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.pos;
        self.pos += 1;
        if n >= self.v.len() {
            return None;
        }
        Some(&self.v[n])
    }
}

impl<'a> ProtoP<'a> {
    pub fn remaining(&self) -> usize {
        self.v.len() - self.pos
    }
}

