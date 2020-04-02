use crate::action::Action;
use crate::ecs_ish::{GenData, GenManager};
use crate::error::ActionError;
use crate::expr::Expr;
use crate::proto::{Proto, ProtoP};
use crate::value::Value;
use gobble::{LCChars, Parser};
//use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::Path;

#[derive(Debug)]
pub struct Base {
    gd: GenData,
    swap_off: bool,
}

#[derive(Debug)]
pub struct Scope {
    bases: Vec<Base>, //swapoff
    gm: GenManager,
}

impl Scope {
    pub fn new(gm: GenManager) -> Scope {
        Scope {
            bases: vec![Base {
                gd: gm.push(Value::map()),
                swap_off: false,
            }],
            gm,
        }
    }

    pub fn on_wrap<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Scope) -> T,
    {
        self.bases.push(Base {
            gd: self.gm.push(Value::map()),
            swap_off: false,
        });
        let res = f(&mut self);
        loop {
            let bas = self.bases.pop().unwrap();
            self.gm.drop_ref(bas.gd);
            if !bas.swap_off {
                return res;
            }
        }
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

    pub fn get<'a>(&'a self, p: &Proto) -> Option<&'a Value> {
        if p.dots == 0 {
            return self.get_from(&self.bases[0].gd, p.pp());
        }
        if p.dots <= self.bases.len() {
            let bpos = self.bases.len() - p.dots;
            return self.get_from(&self.bases[bpos].gd, p.pp());
        }
        None
    }

    pub fn get_from<'a>(&'a self, base: &GenData, mut pp: ProtoP) -> Option<&'a Value> {
        let mut v = self.gm.get(base)?;
        while let Value::Ref(g) = v {
            v = self.gm.get(g)?;
        }
        for p in pp {
            let g = v.child_ref(p)?;
            v = self.gm.get(g)?;
            while let Value::Ref(g) = v {
                v = self.gm.get(g)?;
            }
        }
        Some(v)
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
                    //TODO Set parameters
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

    pub fn set(&mut self, p: &Proto, v: Value) -> Result<Option<Value>, ActionError> {
        unimplemented!()
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
