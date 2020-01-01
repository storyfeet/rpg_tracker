use crate::action::Action;
use crate::error::ActionError;
use crate::expr::Expr;
use crate::parse::ActionReader;
use crate::proto::Proto;
use crate::value::{SetResult, Value};
use std::fmt::Debug;
use std::path::Path;

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

    pub fn eat_data(self) -> Value {
        self.data
    }

    pub fn from_file<P: AsRef<Path> + Debug>(fname: P) -> Result<Self, ActionError> {
        let mut res = Scope::new();
        res.run_file(fname)?;
        Ok(res)
    }
    pub fn run_file<P: AsRef<Path> + Debug>(&mut self, fname: P) -> Result<(), ActionError> {
        let fs = std::fs::read_to_string(&fname).map_err(|e| ActionError::new(&e.to_string()))?;
        let r = ActionReader::new(&fs);

        for a in r {
            let a = match a {
                Ok(v) => v,
                Err(e) => {
                    println!("Error {}", e);
                    continue;
                }
            };
            if let Err(e) = self.do_action(&a.action) {
                println!("Error {} at {}", e, a.line)
            }
        }
        Ok(())
    }

    pub fn get(&self, p: &Proto) -> Option<&Value> {
        let pc = self.in_context(p).ok()?; // CONSIDER: make fn return result
                                           //var with name exists
        if let Some(v) = self.data.get_path(&mut pc.pp()) {
            return Some(v);
        }
        unsafe {
            match self.parent {
                Parent::Mut(par) => (&*par).get(p),
                Parent::Const(par) => (&*par).get(p),
                Parent::None => None,
            }
        }
    }

    pub fn on_wrap<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut Scope) -> T,
    {
        let mut wrap = Scope {
            base: None,
            data: Value::tree(),
            parent: Parent::Const(self as *const Scope),
        };
        f(&mut wrap)
    }

    pub fn call_expr(&self, ex: Expr) -> Result<Value, ActionError> {
        let mut wrap = Scope {
            base: None,
            data: Value::tree(),
            parent: Parent::Const(self as *const Scope),
        };
        ex.eval(&mut wrap)
    }

    pub fn for_each<T, IT>(
        &mut self,
        it: IT,
        fold: Option<Value>,
        func: Value,
    ) -> Result<Option<Value>, ActionError>
    where
        Value: From<T>,
        IT: Iterator<Item = (T, Value)>,
    {
        let (pnames, actions) = match func {
            Value::FuncDef(pnames, actions) => (pnames, actions),
            _ => return Err(ActionError::new("foreach requires a func def")),
        };
        let mut scope = Scope {
            base: None,
            data: Value::tree(),
            parent: Parent::Mut(self as *mut Scope),
        };

        let fold_name = if pnames.len() > 2 { &pnames[2] } else { "fold" };

        if let Some(f) = fold {
            scope.set_param(fold_name, f);
        }

        for (k, v) in it {
            match pnames.len() {
                0 => {
                    scope.set_param("k", Value::from(k));
                    scope.set_param("v", v);
                }
                1 => {
                    scope.set_param(&pnames[0], v);
                }
                _ => {
                    scope.set_param(&pnames[0], Value::from(k));
                    scope.set_param(&pnames[1], v);
                }
            }

            for a in &actions {
                let done = scope.do_action(a);
                match done {
                    Ok(Some(v)) => {
                        scope.set_param(fold_name, v);
                        break;
                    }
                    Err(e) => return Err(e),
                    Ok(None) => {}
                }
            }
        }
        Ok(scope.get(&Proto::one(fold_name)).map(|v| v.clone()))
    }

    pub fn run_func(
        &mut self,
        pnames: &[String],
        actions: &[Action],
        params: &[Value],
    ) -> Result<Option<Value>, ActionError> {
        let mut scope = Scope {
            base: None,
            data: Value::tree(),
            parent: Parent::Mut(self as *mut Scope),
        };

        for p in 0..params.len() {
            if pnames.len() > p {
                scope.set_param(&pnames[p], params[p].clone());
            }
        }

        for a in actions {
            let done = scope.do_action(a);
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

    pub fn set_param(&mut self, k: &str, v: Value) {
        self.data.set_at_path(Proto::one(k).pp(), v);
    }

    fn on_sr(&mut self, sr: SetResult) -> Result<Option<Value>, ActionError> {
        match sr {
            SetResult::Ok(v) => Ok(v),
            SetResult::Deref(p, v) => self.set(&p, v),
            SetResult::Err(e) => Err(e),
        }
    }

    pub fn set(&mut self, p: &Proto, v: Value) -> Result<Option<Value>, ActionError> {
        //proto named var
        if p.var {
            let sr = self.data.set_at_path(p.pp(), v);
            return self.on_sr(sr);
        }

        let p2 = self.in_context(p)?;
        //Try for local variable first
        if let Some(vname) = p2.pp().next() {
            if self.data.has_child(&vname.as_string()) {
                let sr = self.data.set_at_path(p2.pp(), v);
                return self.on_sr(sr);
            }
        }
        //try parent
        unsafe {
            if let Parent::Mut(par) = self.parent {
                return (&mut *par).set(p, v);
            }
        }
        let sr = self.data.set_at_path(p.pp(), v);
        self.on_sr(sr)
    }

    pub fn in_context(&self, p: &Proto) -> Result<Proto, ActionError> {
        //println!("in context p = {}",p);
        match p.dotted {
            false => Ok(p.clone()),
            true => match self.base.as_ref() {
                Some(b) => Ok(b.extend_new(p.pp())),
                None => unsafe {
                    match self.parent {
                        Parent::Const(par) => (&*par).in_context(p),
                        Parent::Mut(par) => (&*par).in_context(p),
                        Parent::None => Err(ActionError::new("Cannot find context for '.'")),
                    }
                },
            },
        }
    }

    pub fn do_action(&mut self, a: &Action) -> Result<Option<Value>, ActionError> {
        fn err(s: &str) -> ActionError {
            ActionError::new(s)
        };
        match a {
            Action::Select(proto_op) => {
                println!("Select");
                if let Some(px) = proto_op {
                    let nbase = px.eval_expr(self)?.as_proto()?.clone();
                    if self.get(&nbase).is_none(){
                        self.set(&nbase,Value::tree())?;
                    }
                    self.base = Some(nbase);
                }else {
                    self.base = None;
                }
            }
            Action::Set(px, v) => {
                let pv = px.eval_expr(self)?;
                let proto = pv.as_proto()?;
                self.set(proto, v.eval(self)?)
                    .map_err(|_| err("Could not Set"))?;
            }
            Action::Add(px, v) => {
                let pv = px.eval_expr(self)?;
                let proto = pv.as_proto()?;
                match self.get(proto) {
                    Some(ov) => {
                        let nv = ov.clone().try_add(v.eval(self)?)?;
                        self.set(proto, nv).map_err(|_| err("Could not Add"))?;
                    }
                    None => {
                        self.set(proto, v.eval(self)?)
                            .map_err(|_| err("Could not add"))?;
                    }
                }
            },
            Action::Sub(px, v) =>{
                let pv = px.eval_expr(self)?;
                let proto = pv.as_proto()?;
                match self.get(proto) {
                    Some(ov) => {
                        let nv = ov.clone().try_sub(v.eval(self)?)?;
                        self.set(proto, nv).map_err(|_| err("Could not sub"))?;
                    }
                    None => {
                        self.set(proto, Expr::neg(v.clone()).eval(self)?)
                            .map_err(|_| err("Coult not sub"))?;
                    }
                }
            },
            Action::Expr(e) => return Ok(Some(e.eval(self)?)),
            Action::Proto(px) => {

                return px.clone().deref(1).eval_mut(self);
            }
        };
        Ok(None)
    }
}
