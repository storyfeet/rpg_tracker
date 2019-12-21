use crate::prev_iter::Backer;
use crate::token::{TokPrev, Token};
use std::fmt::{Display, Formatter};
#[derive(Debug, Clone, PartialEq)]
pub struct Proto {
    pub dots: i32,
    pub derefs: i32,
    v: Vec<String>,
}

impl Display for Proto {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for _ in 0..self.derefs {
            write!(f, "$")?;
        }
        for _ in 0..self.dots {
            write!(f, ".")?;
        }
        for (i, s) in self.v.iter().enumerate() {
            if i != 0 {
                write!(f, ".")?;
            }
            write!(f, "{}", s.replace(".","\\."))?;
        }
        Ok(())
    }
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
        let mut t = TokPrev::new(s);
        Self::from_tokens(&mut t)
    }

    pub fn join_on_dot(&self, p: Self) -> Self {
        if p.dots <= 0 {
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

    pub fn from_str(s: &str) -> Self {
        let mut tp = TokPrev::new(s);
        Self::from_tokens(&mut tp)
    }

    pub fn from_tokens(t: &mut TokPrev) -> Self {
        let mut res = Proto::empty(0);
        while let Some(v) = t.next() {
            match v {
                Token::Dot => {
                    if res.v.len() == 0 {
                        res.dots += 1
                    }
                }
                Token::Dollar => {
                    if res.v.len() == 0 {
                        res.derefs += 1
                    }
                }
                Token::Qoth(s) | Token::Ident(s) => res.v.push(s),
                Token::Num(n) => res.v.push(n.to_string()),
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
        ProtoP {
            v: &self.v,
            pos: 0,
            stop: self.v.len(),
        }
    }

    pub fn extend(&mut self, pp: ProtoP) {
        self.v.extend(pp.map(|s| s.to_string()))
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
    v: &'a Vec<String>,
    pos: usize,
    stop: usize,
}

impl<'a> Iterator for ProtoP<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.pos;
        self.pos += 1;
        if n >= self.stop {
            return None;
        }
        Some(&self.v[n])
    }
}

impl<'a> ProtoP<'a> {
    pub fn remaining(&self) -> usize {
        if self.pos > self.v.len() {
            return 0;
        }
        self.v.len() - self.pos
    }
    pub fn parent(&self) -> Self {
        let mut res = self.clone();
        if res.stop >= 1 {
            res.stop -= 1;
        }
        res
    }
}
