
#[derive(Debug,Clone)]
pub struct Proto(
    Vec<String>
);

impl Proto {
    pub fn empty()->Self{
        Proto(Vec::new())
    }
    pub fn new(s:&str)->Self{
        let ss = match s.chars().next(){
            Some('$')=> &s[1..],
            None=>return Self::empty(),
            _ => s,
        };
        if ss.len() == 0 {
            return Self::empty();
        }
        Proto(s.split(".").map(|s|s.to_string()).collect())
    }

    pub fn push(&mut self,s:&str){
        self.0.push(s.to_string())
    }

    pub fn pp<'a>(&'a self)->ProtoP<'a>{
        ProtoP{
            v:&self.0,
            pos:0,
        }
    }

    pub fn extend(&mut self,pp :ProtoP){
        self.0.extend(pp.map(|s|s.to_string()))
    }

}


pub struct ProtoP<'a>{
    v:&'a Vec<String>,
    pos:usize,
}

impl<'a> Iterator for ProtoP<'a>{
    type Item = &'a str;
    fn next(&mut self)->Option<Self::Item>{
        let n = self.pos;
        self.pos +=1;
        if n  >= self.v.len(){
            return None
        }
        Some(&self.v[n])
    }
}

impl <'a> ProtoP<'a>{
    pub fn remaining(&self)->usize{
        self.v.len()- self.pos
    }
}
