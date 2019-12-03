use crate::action::Action;
use crate::error::ActionError;
use crate::proto::{Proto, ProtoP};
use crate::value::Value;
use std::collections::BTreeMap;
use crate::stack::StackItem;

#[derive(Debug)]
pub struct DnData {
    stack: Vec<StackItem>,
    pub data: Value,
}

///stack len cannot be zero if used correctly, pop will not remove last element stack is not to
///be public
impl DnData {
    pub fn new() -> Self {
        DnData {
            stack: vec![StackItem::new(Proto::empty(0))],
            data: Value::Tree(BTreeMap::new()),
        }
    }

    pub fn push_save(&mut self,p:Proto) {
        self.stack.push(StackItem::new(p))
    }
    pub fn restore(&mut self) {
        match self.stack.len(){
            0|1=>{},
            _=>{self.stack.pop();}
        }
    }
    pub fn set_curr(&mut self,p:Proto){
        let lpos = self.stack.len()-1;
        self.stack[lpos].set_curr(p);
    }

    pub fn get_pp(&self, p: ProtoP) -> Option<&Value> {
        let mut p2 = p.clone();
        //begins with var
        let lpos = self.stack.len()-1;
        if let Some("var") = p2.next() {
            return self.stack[lpos].vars.get_path(p2);
        }
        //var with name exists
        let p2 = p.clone();
        if let Some(v) = self.stack[lpos].vars.get_path(p2){
            return Some(v);
        }
        
        self.data.get_path(p)
    }

    pub fn set_param(&mut self, k: &str, v: Value) {
        let ln = self.stack.len();
        self.stack[ln - 1]
            .vars
            .set_at_path(Proto::one(k, 0).pp(), v)
            .ok();
    }

    pub fn set_pp(&mut self, p: ProtoP, v: Value) -> Result<Option<Value>, ()> {
        let lpos = self.stack.len()-1;
        //proto named var
        let mut p2 = p.clone();
        if let Some("var") = p2.next() {
            return self.stack[lpos].vars.set_at_path(p2, v);
        }
        //var exists with name
        let mut p2 = p.clone();
        if let Some(vname) = p2.next(){
            if self.stack[lpos].vars.has_child(vname){
                let p3 = p.clone();
                return self.stack[lpos].vars.set_at_path(p3,v);
                
            }
        }
        self.data.set_at_path(p, v)
    }

    pub fn in_context(&self,p:&Proto)->Proto{
        let lpos = self.stack.len()-1;
        self.stack[lpos].in_context(p)
    }

    pub fn do_action(&mut self, a: Action) -> Result<Option<Value>, failure::Error> {
        fn err(s: &str) -> ActionError {
            ActionError::new(s)
        };
        match a {
            Action::Select(proto) => {
                let np = self.in_context(&proto);
                match self.get_pp(np.pp()) {
                    Some(_) => {}
                    _ => {
                        self.data
                            .set_at_path(np.pp(), Value::tree())
                            .map_err(|_| err("count not Create object for selct"))?;
                    }
                }
                self.set_curr(np);
            }
            Action::Set(proto, v) => {
                let np = self.in_context(&proto);
                self.set_pp(np.pp(), v)
                    .map_err(|_| err("Could not Set"))?;
            }
            Action::Add(proto, v) => {
                let np = self.in_context(&proto);
                match self.get_pp(np.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_add(v)?;
                        self.set_pp(np.pp(), nv).map_err(|_| err("Could not Add"))?;
                    }
                    None => {
                        self.set_pp(np.pp(), v).map_err(|_| err("Coult not add"))?;
                    }
                }
            }
            Action::Sub(proto, v) => {
                let np = self.in_context(&proto);
                match self.get_pp(np.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_sub(v)?;
                        self.set_pp(np.pp(), nv).map_err(|_| err("Could not Add"))?;
                    }
                    None => {
                        self.set_pp(np.pp(), v).map_err(|_| err("Coult not add"))?;
                    }
                }
            }
            Action::Expr(e) => return Ok(Some(e.eval(self)?)),
            Action::CallFunc(proto, params) => {
                //TODO work out how to pass params
                let np = self.in_context(&proto);
                let (pnames,actions) = match self.get_pp(np.pp()){
                    Some(Value::FuncDef(pn,ac)) =>
                        (pn.clone(),ac.clone()),
                    _=>return Err(err("func on notafunc").into()),

                };
                
                self.push_save(np);
                for p in 0..params.len(){
                    if pnames.len() > p{
                        self.set_param(&pnames[p],params[p].clone());
                    }
                }

                for a in actions {
                    println!("func action {:?}", a);
                    match self.do_action(a){
                        Ok(Some(v))=>{
                            self.restore();
                            return Ok(Some(v))
                        }
                        Err(e)=>return Err(e),
                        Ok(None)=>{},
                    }
                }
                self.restore();

            }
        };
        Ok(None)
    }
}
