use crate::error::ActionError;
use crate::value::Value;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoNode {
    Num(isize),
    Str(String),
}

impl ProtoNode {
    pub fn str(s: &str) -> Self {
        ProtoNode::Str(s.to_string())
    }

    pub fn as_string(&self) -> String {
        match self {
            ProtoNode::Num(n) => n.to_string(),
            ProtoNode::Str(s) => s.clone(),
        }
    }

    pub fn as_num(&self) -> Option<isize> {
        match self {
            ProtoNode::Num(n) => Some(*n),
            ProtoNode::Str(s) => isize::from_str(s).ok(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proto {
    v: Vec<ProtoNode>,
    pub dot: bool,
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
            }
        }
        Ok(())
    }
}

impl Proto {
    pub fn new() -> Self {
        Proto {
            v: Vec::new(),
            dot: false,
        }
    }

    pub fn one(s: &str) -> Self {
        Proto {
            v: vec![ProtoNode::str(s)],
            dot: false,
        }
    }

    pub fn num(n: isize) -> Self {
        Proto {
            v: vec![ProtoNode::Num(n)],
            dot: false,
        }
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

    pub fn push_val(&mut self, v: Value) -> Result<(), ActionError> {
        match v {
            Value::Str(s) => self.v.push(ProtoNode::Str(s)),
            Value::Num(n) => self.v.push(ProtoNode::Num(n)),
            _ => return Err(ActionError::new("proto parts must resolve to str or num")),
        }
        Ok(())
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
