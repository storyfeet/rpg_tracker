use crate::action::Action;
use crate::ecs_ish::{GenData, GenManager};
use crate::error::ActionError;
use crate::expr::Expr;
use crate::proto::Proto;
use crate::value::Value;
use gobble::{LCChars, Parser};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::Path;

//unsafe invariant that must be maintained:
//rescoped Scopes can only be used for immediate function calls

#[derive(Debug)]
pub struct Scope {
    base: Vec<GenData>,
    gm: GenManager,
}

impl Scope {
    pub fn new(gm: GenManager) -> Scope {
        Scope {
            base: vec![gm.push(Value::map())],
            gm,
        }
    }

    pub fn on_wrap<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Scope) -> T,
    {
        self.base.push(self.gm.push(Value::map()));
        let res = f(&mut self);
        let b = self.base.pop().unwrap();
        self.gm.dec_rc(b);
        res
    }

    pub fn handle_input(&mut self, s: &str) -> Result<(), ActionError> {
        let mut ss = LCChars::str(s);
        let ac = crate::nomp::action();
        loop {
            match ac.parse(&ss) {
                Ok((ns, a)) => {
                    ss = ns;
                    match self.do_action(&a) {
                        //TODO consider writing file
                        Ok(Some(v)) => {
                            println!("{}", v.print(0, &self.gm));
                        }
                        Ok(None) => {}
                        Err(e) => println!("Error {}", e),
                    }
                }
                Err(e) => println!("Error {}", e),
            }
        }
    }

    pub fn gm_mut(&mut self) -> &mut GenManager {
        &mut self.gm
    }

    pub fn run_file<P: AsRef<Path> + Debug>(&mut self, fname: P) -> Result<(), ActionError> {
        let fs = std::fs::read_to_string(&fname).map_err(|e| ActionError::new(&e.to_string()))?;
        self.handle_input(&fs)
    }

    pub fn push_mem(&mut self, v: Value) -> GenData {
        self.gm.push(v)
    }

    pub fn as_ref(&self, p: Proto) -> Option<&GenData> {
        let mut gd = self.base.get(self.base.len() - 1);
        for pp in p.pp() {
            match self.gm.get(&gd) {
                None => return None,
                Some(Value::Map(m)) => gd = m.get(&pp.as_string())?.clone_weak(),
                Some(Value::List(l)) => gd = l.get(&pp.as_num().ok()?),
            }
        }
    }

    pub fn call_expr(&self, ex: Expr) -> Result<Value, ActionError> {
        self.on_wrap(|sc| ex.eval(sc))
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
        unimplemented!()
    }

    pub fn run_func(
        &mut self,
        pnames: &[String],
        actions: &[Action],
        params: Vec<Value>,
    ) -> Result<Option<Value>, ActionError> {
        self.on_wrap(|sc| {
            for p in 0..params.len() {
                if pnames.len() > p {
                    sc.set_local(&pnames[p], params[p]);
                }
            }
            for a in actions {
                let done = sc.do_action(a);
                match done {
                    Ok(Some(v)) => {
                        return Ok(Some(v));
                    }
                    Err(e) => return Err(e),
                    Ok(None) => {}
                }
            }
            Ok(None)
        })
    }

    pub fn set_local(&mut self, k: &str, v: Value) {
        if let Some(ref b) = self.base.get(self.base.len() - 1) {
            if let Some(Value::Map(ref mut m)) = self.gm.get_mut(b) {
                self.set_child(m, k, v)
            }
        }
    }

    pub fn set_child(&mut self, par: &mut BTreeMap<String, GenData>, k: &str, child: Value) {
        let ng = self.gm.push(child);
        match par.insert(k.to_string(), ng) {
            Some(gd) => self.gm.dec_rc(gd),
            None => {}
        }
    }

    pub fn set(&mut self, p: &Proto, v: Value) -> Result<Option<Value>, ActionError> {
        //proto named var

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

    pub fn do_action(&mut self, a: &Action) -> Result<Option<Value>, ActionError> {
        fn err(s: &str) -> ActionError {
            ActionError::new(s)
        };
        match a {
            Action::Select(ex) => {
                println!("Select");
                if let Some(px) = proto_op {
                    let nbase = px.eval_expr(self)?.as_proto()?.clone();
                    if self.get(&nbase).is_none() {
                        self.set(&nbase, Value::map())?;
                    }
                    self.base = Some(nbase);
                } else {
                    self.base = None;
                }
            }
            Action::Set(px, v) => {
                let proto = px.eval_path(self)?;
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
            }
            Action::Sub(px, v) => {
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
            }
            Action::Expr(e) => return Ok(Some(e.eval(self)?)),
            Action::Proto(px) => {
                return px.clone().deref(1).eval_mut(self);
            }
        };
        Ok(None)
    }
}
