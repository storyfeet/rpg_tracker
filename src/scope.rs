use crate::action::Action;
use crate::error::ActionError;
use crate::proto::{Proto, ProtoP};
use crate::value::{SetResult, Value};
use std::fmt::Debug;

pub trait Scoper:Debug+Sized{
    fn rescope<'a>(&'a mut self, this: Proto) -> Scope<'a> {
        let mut t = Value::tree();
        t.set_at_path(Proto::one("this", 0).pp(), Value::Proto(this));
        Scope {
            data: t,
            base: None,
            parent: Some(self),
        }
    }
}

#[derive(Debug)]
pub struct Scope<'a>{
    data: Value,
    base: Option<Proto>,
    parent: Option<&'a dyn Scoper>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Scope<'static> {
        let t = Value::tree();
        Scope {
            data: t,
            base: None,
            parent: None,
        }
    }



    pub fn get_pp(&self, p: ProtoP) -> Option<&Value> {
        //var with name exists
        let mut p2 = p.clone();
        if let Some(v) = self.data.get_path(&mut p2) {
            if p2.remaining() > 0 {
                if let Value::Proto(v2) = v {
                    let np = v2.extend_new(p2);
                    return self.get_pp(np.pp());
                }
            }
            return Some(v);
        }
        if let Some(ref par) = self.parent {
            return par.get_pp(p);
        }
        None
    }

    fn on_sr(&mut self, sr: SetResult) -> Result<Option<Value>, ActionError> {
        match sr {
            SetResult::Ok(v) => return Ok(v),
            SetResult::Deref(p, v) => return self.set_pp(p.pp(), v),
            SetResult::Err(e) => return Err(e),
        }
    }
    pub fn set_param(&mut self, k: &str, v: Value) {
        self.data.set_at_path(Proto::one(k, 0).pp(), v);
    }
    pub fn set_pp(&mut self, p: ProtoP, v: Value) -> Result<Option<Value>, ActionError> {
        //proto named var
        let mut p2 = p.clone();
        if let Some("var") = p2.next() {
            let sr = self.data.set_at_path(p2, v);
            return self.on_sr(sr);
        }
        //Try for local variable first
        let mut p2 = p.clone();
        if let Some(vname) = p2.next() {
            if self.data.has_child(vname) {
                let p3 = p.clone();
                let sr = self.data.set_at_path(p3, v);
                return self.on_sr(sr);
            }
        }
        //try parent
        if let Some(par) = &mut self.parent {
            return par.set_pp(p, v);
        }

        let sr = self.data.set_at_path(p, v);
        self.on_sr(sr)
    }

    pub fn in_context(&self, p: &Proto) -> Result<Proto, ActionError> {
        let mut res = match p.dots {
            0 => p.clone(),
            _ => match & self.parent {
                Some(par) => return par.in_context(p),
                None => return Err(ActionError::new("Cannot find context for '.'")),
            },
        };
        let dcount = res.derefs;
        for _ in 0..dcount {
            println!("dereffing");
            if let Some(Value::Proto(der)) = self.get_pp(res.pp()) {
                res = der.clone();
            }
        }
        Ok(res)
    }
    pub fn resolve(&'a self, v: Value) -> Result<Value, ActionError> {
        match v {
            Value::Ex(e) => e.eval(self),
            Value::Proto(mut p) => {
                let dc = p.derefs;
                for i in 0..dc {
                    match self.get_pp(p.pp()) {
                        Some(Value::Proto(np)) => p = np.clone(),
                        Some(v) => {
                            if i + 1 == dc {
                                return Ok(v.clone());
                            } else {
                                return Err(ActionError::new("deref beyond protos"));
                            }
                        }
                        None => return Err(ActionError::new("deref to nothing")),
                    }
                }
                Ok(Value::Proto(p))
            }
            _ => Ok(v),
        }
    }

    pub fn do_action(&mut self, a: Action) -> Result<Option<Value>, ActionError> {
        fn err(s: &str) -> ActionError {
            ActionError::new(s)
        };
        match a {
            Action::Select(proto) => {
                let np = self.in_context(&proto)?;
                match self.get_pp(np.pp()) {
                    Some(_) => {}
                    _ => {
                        self.set_pp(np.pp(), Value::tree())
                            .map_err(|_| err("count not Create object for selct"))?;
                    }
                }
                self.base = Some(np.clone());
            }
            Action::Set(proto, v) => {
                let np = self.in_context(&proto)?;
                self.set_pp(np.pp(), self.resolve(v)?)
                    .map_err(|_| err("Could not Set"))?;
            }
            Action::Add(proto, v) => {
                let np = self.in_context(&proto)?;
                match self.get_pp(np.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_add(self.resolve(v)?)?;
                        self.set_pp(np.pp(), nv).map_err(|_| err("Could not Add"))?;
                    }
                    None => {
                        self.set_pp(np.pp(), self.resolve(v)?)
                            .map_err(|_| err("Coult not add"))?;
                    }
                }
            }
            Action::Sub(proto, v) => {
                let np = self.in_context(&proto)?;
                match self.get_pp(np.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_sub(self.resolve(v)?)?;
                        self.set_pp(np.pp(), nv).map_err(|_| err("Could not Add"))?;
                    }
                    None => {
                        self.set_pp(np.pp(), self.resolve(v.try_neg()?)?)
                            .map_err(|_| err("Coult not add"))?;
                    }
                }
            }
            Action::Expr(e) => return Ok(Some(e.eval(self)?)),
            Action::CallFunc(proto, params) => {
                //TODO work out how to pass params
                let np = self.in_context(&proto)?;
                let nparent = np.parent();
                let (pnames, actions) = match self.get_pp(np.pp()) {
                    Some(Value::FuncDef(pn, ac)) => (pn.clone(), ac.clone()),
                    _ => return Err(err("func on notafunc").into()),
                };

                let mut wrap = self.rescope(nparent);
                for p in 0..params.len() {
                    if pnames.len() > p {
                        wrap.set_param(&pnames[p], params[p].clone());
                    }
                }

                for a in actions {
                    let done = wrap.do_action(a);
                    match done{
                        Ok(Some(v)) => {
                            return Ok(Some(v));
                        }
                        Err(e) => return Err(e),
                        Ok(None) => {}
                    }
                }
            }
        };
        Ok(None)
    }
}
