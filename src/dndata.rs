use crate::action::Action;
use crate::error::ActionError;
use crate::proto::{Proto, ProtoP, ProtoStack};
use crate::value::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct DnData {
    stack: ProtoStack,
    pub data: Value,
    locals: Vec<Value>,
}

impl DnData {
    pub fn new() -> Self {
        DnData {
            stack: ProtoStack::new(),
            data: Value::Tree(BTreeMap::new()),
            locals: Vec::new(),
        }
    }

    pub fn save(&mut self) {
        self.stack.save();
        self.locals.push(Value::tree());
    }
    pub fn restore(&mut self) {
        self.stack.restore();
        self.locals.pop();
    }

    pub fn get_pp(&self, p: ProtoP) -> Option<&Value> {
        if self.locals.len() == 0 {
            return self.data.get_path(p);
        }
        let mut p2 = p.clone();
        if let Some("var") = p2.next() {
            let ln = self.locals.len();
            return self.locals[ln - 1].get_path(p2);
        }
        self.data.get_path(p)
    }

    pub fn set_param(&mut self, k: &str, v: Value) {
        let ln = self.locals.len();
        self.locals[ln - 1]
            .set_at_path(Proto::one(k, 0).pp(), v)
            .ok();
    }

    pub fn set_pp(&mut self, p: ProtoP, v: Value) -> Result<Option<Value>, ()> {
        if self.locals.len() == 0 {
            return self.data.set_at_path(p, v);
        }
        let mut p2 = p.clone();
        if let Some("var") = p2.next() {
            let ln = self.locals.len();
            return self.locals[ln - 1].set_at_path(p2, v);
        }
        self.data.set_at_path(p, v)
    }

    pub fn do_action(&mut self, a: Action) -> Result<Option<Value>, failure::Error> {
        fn err(s: &str) -> ActionError {
            ActionError::new(s)
        };
        match a {
            Action::Select(proto) => {
                let np = self.stack.in_context(proto);
                match self.get_pp(np.pp()) {
                    Some(_) => {}
                    _ => {
                        self.data
                            .set_at_path(np.pp(), Value::tree())
                            .map_err(|_| err("count not Create object for selct"))?;
                    }
                }
                self.stack.set_curr(np);
            }
            Action::Set(proto, v) => {
                let np = self.stack.in_context(proto);
                self.set_pp(np.pp(), v)
                    .map_err(|_| err("Could not Set"))?;
            }
            Action::Add(proto, v) => {
                let np = self.stack.in_context(proto);
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
                let np = self.stack.in_context(proto);
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
                let np = self.stack.in_context(proto);
                let (pnames,actions) = match self.get_pp(np.pp()){
                    Some(Value::FuncDef(pn,ac)) =>
                        (pn.clone(),ac.clone()),
                    _=>return Err(err("func on notafunc").into()),

                };
                
                self.save();
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
