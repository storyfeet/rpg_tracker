use crate::error::ActionError;
use crate::value::Value;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum ProtoNode {
    Num(usize),
    Str(String),
    Deref,
}

impl ProtoNode {
    pub fn str(s: &str) -> Self {
        ProtoNode::Str(s.to_string())
    }

    pub fn as_string(&self) -> String {
        match self {
            ProtoNode::Num(n) => n.to_string(),
            ProtoNode::Str(s) => s.clone(),
            ProtoNode::Deref => String::new(),
        }
    }

    pub fn as_num(&self) -> Option<usize> {
        match self {
            ProtoNode::Num(n) => Some(*n),
            ProtoNode::Str(s) => usize::from_str(s).ok(),
            ProtoNode::Deref => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proto {
    v: Vec<ProtoNode>,
    pub dots: usize,
    pub root: bool,
}

impl Display for Proto {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for (i, node) in self.v.iter().enumerate() {
            if i != 0 {
                write!(f, ".")?;
            }
            match node {
                ProtoNode::Num(n) => write!(f, "{}", n)?,
                ProtoNode::Str(s) => write!(f, "{}", s.replace(".", "\\."))?,
                ProtoNode::Deref => write!(f, "DEREF")?,
            }
        }
        Ok(())
    }
}

impl Proto {
    pub fn new() -> Self {
        Proto {
            v: Vec::new(),
            dots: 0,
            root: false,
        }
    }

    pub fn push(mut self, pn: ProtoNode) -> Self {
        self.v.push(pn);
        self
    }

    pub fn rooted(mut self) -> Self {
        self.root = true;
        self
    }

    pub fn one(pn: ProtoNode) -> Self {
        Self::new().push(pn)
    }
    pub fn str(s: &str) -> Self {
        Self::one(ProtoNode::Str(s.to_string()))
    }

    pub fn num(n: usize) -> Self {
        Self::one(ProtoNode::Num(n))
    }
    pub fn dr() -> Self {
        Self::one(ProtoNode::Deref)
    }

    pub fn dot(mut self) -> Self {
        self.dots += 1;
        self
    }

    pub fn as_api_func_name(&self) -> Option<&str> {
        if self.v.len() > 1 {
            return None;
        }
        match self.v.get(0) {
            Some(ProtoNode::Str(s)) => Some(s),
            _ => None,
        }
    }

    pub fn parent(&self) -> Self {
        let mut res = self.clone();
        if res.v.len() > 0 {
            res.v.pop();
        }
        res
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
}

#[derive(Clone, Debug)]
pub struct ProtoP<'a> {
    p: &'a Proto,
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

    pub fn parent(&self) -> Self {
        let mut res = self.clone();
        if res.stop >= 1 {
            res.stop -= 1;
        }
        res
    }
}
