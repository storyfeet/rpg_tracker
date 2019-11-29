use crate::prev_iter::{Prev,LineCounter,Backer};
use crate::token::{Token, Tokenizer};
#[derive(Debug, Clone, PartialEq)]
pub struct Proto {
    pub dot: bool,
    v: Vec<String>,
}

impl Proto {
    pub fn empty(dot: bool) -> Self {
        Proto { dot, v: Vec::new() }
    }
    pub fn one(s: &str, dot: bool) -> Self {
        let mut v = Vec::new();
        v.push(s.to_string());
        Proto { dot, v }
    }
    pub fn new(s: &str) -> Self {
        let mut t = Prev::new(Tokenizer::new(s));
        Self::from_tokens(&mut t)
    }

    pub fn from_tokens<T:Iterator<Item=Token>+Backer+LineCounter>(t: &mut T) -> Self {
        let mut res = Proto::empty(false);
        match t.next() {
            Some(Token::Dot) => res.dot = true,
            _ => t.back(),
        }
        while let Some(v) = t.next() {
            match v {
                Token::Dot => {}
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

    pub fn extend_if_dot(&self, p: &Proto) -> Self {
        if p.dot {
            return self.extend_new(p.pp());
        }
        p.clone()
    }
}

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
