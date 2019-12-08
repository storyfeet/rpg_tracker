use crate::action::Action;
use crate::error::ActionError;
use crate::expr::Expr;
use crate::proto::{Proto, ProtoP};
use crate::value::{SetResult, Value};
use std::fmt::Debug;

//unsafe invariant that must be maintained:
//rescoped Scopes can only be used for immediate function calls

#[derive(Debug)]
pub struct Scope {
    data: Value,
    base: Option<Proto>,
    parent: Parent,
}

#[derive(Debug)]
enum Parent {
    Mut(*mut Scope),
    Const(*const Scope),
    None,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            data: Value::tree(),
            base: None,
            parent: Parent::None,
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

        unsafe {
            match self.parent {
                Parent::Mut(par) => return (&*par).get_pp(p),
                Parent::Const(par) => return (&*par).get_pp(p),
                Parent::None => {}
            }
        }
        None
    }
    pub fn call_func_const(
        &self,
        proto: Proto,
        params: &[Expr],
    ) -> Result<Option<Value>, ActionError> {
        let mut wrap = Scope {
            base: None,
            data: Value::tree(),
            parent: Parent::Const(self as *const Scope),
        };
        wrap.run_func(proto, params)
    }

    pub fn call_func_mut(
        &mut self,
        proto: Proto,
        params: &[Expr],
    ) -> Result<Option<Value>, ActionError> {
        let mut wrap = Scope {
            base: None,
            data: Value::tree(),
            parent: Parent::Mut(self as *mut Scope),
        };
        wrap.run_func(proto, params)
    }

    fn run_func(&mut self, proto: Proto, params: &[Expr]) -> Result<Option<Value>, ActionError> {
        //println!("run_func -{:?}", self.get_pp(Proto::one("self", 0).pp()));
        let np = self.in_context(&proto)?;
        let nparent = np.parent();
        let (pnames, actions) = match self.get_pp(np.pp()) {
            Some(Value::FuncDef(pn, ac)) => (pn.clone(), ac.clone()),
            _ => return Err(ActionError::new("func on notafunc")),
        };

        for p in 0..params.len() {
            if pnames.len() > p {
                let v = params[p].eval(self)?;
                self.set_param(&pnames[p], v);
            }
        }
        if nparent.pp().next() != Some("self") {
            self.set_param("self", Value::Proto(nparent));
        }

        for a in &actions {
            let done = self.do_action(a);
            match done {
                Ok(Some(v)) => {
                    return Ok(Some(v));
                }
                Err(e) => return Err(e),
                Ok(None) => {}
            }
        }
        Ok(None)
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
        unsafe {
            if let Parent::Mut(par) = self.parent {
                return (&mut *par).set_pp(p, v);
            }
        }

        let sr = self.data.set_at_path(p, v);
        self.on_sr(sr)
    }

    pub fn in_context(&self, p: &Proto) -> Result<Proto, ActionError> {
        let mut res = match p.dots {
            0 => p.clone(),
            _ => match self.base.as_ref() {
                Some(b) => b.extend_new(p.pp()),
                None => unsafe {
                    match self.parent {
                        Parent::Const(par) => return (&*par).in_context(p),
                        Parent::Mut(par) => return (&*par).in_context(p),
                        Parent::None => {
                            return Err(ActionError::new("Cannot find context for '.'"))
                        }
                    }
                },
            },
        };
        let dcount = res.derefs;
        for _ in 0..dcount {
            //println!("dereffing");
            if let Some(Value::Proto(der)) = self.get_pp(res.pp()) {
                res = der.clone();
            }
        }
        Ok(res)
    }

    pub fn do_action(&mut self, a: &Action) -> Result<Option<Value>, ActionError> {
        fn err(s: &str) -> ActionError {
            ActionError::new(s)
        };
        match a {
            Action::Show(proto) => {
                let np = self.in_context(&proto)?;
                match self.get_pp(np.pp()) {
                    Some(v) => println!("{}", v.print(0)),
                    None => println!("Empty"),
                }
            }
            Action::Select(proto_op) => {
                if let Some(proto) = proto_op {
                    let np = self.in_context(&proto)?;
                    match self.get_pp(np.pp()) {
                        Some(_) => {}
                        _ => {
                            self.set_pp(np.pp(), Value::tree())
                                .map_err(|_| err("count not Create object for selct"))?;
                        }
                    }
                    self.base = Some(np);
                    return Ok(None);
                }
                self.base = None;
            }
            Action::Set(proto, v) => {
                let np = self.in_context(proto)?;
                self.set_pp(np.pp(), v.eval(self)?)
                    .map_err(|_| err("Could not Set"))?;
            }
            Action::Add(proto, v) => {
                let np = self.in_context(proto)?;
                match self.get_pp(np.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_add(v.eval(self)?)?;
                        self.set_pp(np.pp(), nv).map_err(|_| err("Could not Add"))?;
                    }
                    None => {
                        self.set_pp(np.pp(), v.eval(self)?)
                            .map_err(|_| err("Coult not add"))?;
                    }
                }
            }
            Action::Sub(proto, v) => {
                let np = self.in_context(&proto)?;
                match self.get_pp(np.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_sub(v.eval(self)?)?;
                        self.set_pp(np.pp(), nv).map_err(|_| err("Could not sub"))?;
                    }
                    None => {
                        self.set_pp(np.pp(), Expr::neg(v.clone()).eval(self)?)
                            .map_err(|_| err("Coult not sub"))?;
                    }
                }
            }
            Action::Expr(e) => return Ok(Some(e.eval(self)?)),
            Action::CallFunc(proto, params) => {
                return self.call_func_mut(proto.clone(), &params);
            }
        };
        Ok(None)
    }
}
