use std::fmt::{Display, Formatter};
use crate::value::Value;
use crate::error::ActionError;

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoNode {
    Num(i32),
    Str(String),
}

impl ProtoNode {
    pub fn str(s: &str) -> Self {
        ProtoNode::Str(s.to_string())
    }

    pub fn as_string(&self)->String{
        match self{
            ProtoNode::Num(n)=>n.to_string(),
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
                ProtoNode::Str(s) => write!(f, "{}", s.replace(".", "\\."))?,
            }
        }
        Ok(())
    }
}

impl Proto {
    pub fn new() -> Self {
        Proto {
            dotted:false,
            derefs: 0,
            var:false,
            v: Vec::new(),
        }
    }
    pub fn one(s: &str) -> Self {
        Proto {
            dotted:false,
            var:false,
            v: vec![ProtoNode::str(s)],
            derefs: 0,
        }
    }

    pub fn dot(self) ->Self{
        self.dotted = true;
        self
    }
    pub fn var(self) -> Self {
        self.var = true;
        self
    }
    pub fn deref(self, n:i32)->Self{
        self.derefs += n;
        self
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



    pub fn push_val(&mut self, v: Value)->Result<(),ActionError> {
        match v {
            Value::Str(s)=>self.v.push(ProtoNode::Str(s)),
            Value::Num(n)=>self.v.push(ProtoNode::Num(n)),
            e=>return Err(ActionError::new("proto parts must resolve to str or num"))
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
