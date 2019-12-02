use crate::prev_iter::{Backer, LineCounter, Prev};
use crate::token::{Token, Tokenizer};
#[derive(Debug, Clone, PartialEq)]
pub struct Proto {
    pub dots: i32,
    v: Vec<String>,
}

impl Proto {
    pub fn empty(dots: i32) -> Self {
        Proto {
            dots,
            v: Vec::new(),
        }
    }
    pub fn one(s: &str, dots: i32) -> Self {
        let mut v = Vec::new();
        v.push(s.to_string());
        Proto { dots, v }
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

#[derive(Debug)]
pub struct ProtoStack {
    stack: Vec<Proto>,
    bstack: Vec<usize>,
}

impl ProtoStack {
    pub fn new() -> Self {
        ProtoStack {
            stack: Vec::new(),
            bstack: Vec::new(),
        }
    }
    pub fn set_curr(&mut self, p: Proto) {
        let r = self.in_context(p);
        self.stack.push(r);
    }

    pub fn save(&mut self) {
        self.bstack.push(self.stack.len() - 1);
    }

    pub fn restore(&mut self) {
        if let Some(n) = self.bstack.pop() {
            self.stack.split_off(n);
        }
    }

    pub fn in_context(&mut self, p: Proto) -> Proto {
        if self.stack.len() == 0 {
            return p;
        }
        let base = match p.dots {
            0 => return p,
            1 => self.curr(),
            2 => {
                if let Some(n) = self.bstack.get(self.bstack.len() - 1) {
                    &self.stack[*n]
                } else {
                    &self.stack[0]
                }
            }
            _ => &self.stack[0],
        };
        base.extend_new(p.pp())
    }

    pub fn curr(&mut self) -> &Proto {
        if self.stack.len() == 0 {
            self.stack.push(Proto::empty(0));
        }
        &self.stack[self.stack.len() - 1]
    }
}