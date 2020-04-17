use crate::action::{AcResult, AcReturn, Action};
use crate::ecs_ish::{GenData, GenManager};
use crate::error::ActionError;
use crate::expr::Expr;
use crate::proto::{Proto, ProtoNode, ProtoP};
use crate::value::Value;
//use gobble::{LCChars, Parser};
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
    pub fn new() -> Scope {
        let mut gm = GenManager::new();
        let gd = gm.push(Value::map());
        Scope {
            bases: vec![Base {
                gd,
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
        let res = f(self);
        loop {
            let bas = self.bases.pop().unwrap();
            self.gm.drop_ref(bas.gd);
            if !bas.swap_off {
                return res;
            }
        }
    }

    pub fn handle_input(&mut self, s: &str) -> Result<(), ActionError> {
        //let mut ss = LCChars::str(s);
        use crate::nomp::*;
        use gobble::*;
        let ac = sep_until(maybe(pp_action), l_break(), eoi);
        let v: Vec<Action> = ac.parse_s(s)?.into_iter().filter_map(|a| a).collect();
        for a in v {
            match self.do_action(&a) {
                //TODO consider writing file
                Ok(Value::Null) => {}
                Ok(v) => {
                    println!("{}", v.print(0, &self.gm));
                }
                Err(e) => println!("Error {}", e),
            }
        }
        Ok(())
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

    pub fn select_base(&self, p: &Proto) -> Option<&GenData> {
        let n = if p.root {
            p.dots
        } else {
            let mut last = 0;
            for (i, b) in self.bases.iter().enumerate() {
                if !b.swap_off {
                    last = i;
                }
            }
            last + p.dots
        };
        self.bases.get(n).map(|v| &v.gd)
    }

    pub fn get<'a>(&'a self, p: &Proto) -> Option<&'a Value> {
        let b = self.select_base(p)?.clone_weak();
        self.get_from(&b, p.pp()).map(|(_, v)| v)
    }

    pub fn get_ref(&self, p: &Proto) -> Option<GenData> {
        let b = self.select_base(p)?.clone_weak();
        self.get_from(&b, p.pp()).map(|(r, _)| r)
    }

    pub fn get_from<'a>(&'a self, base: &GenData, pp: ProtoP) -> Option<(GenData, &'a Value)> {
        let mut v = self.gm.get(base)?;
        let mut lg = base;
        while let Value::Ref(g) = v {
            lg = g;
            v = self.gm.get(g)?;
        }
        for p in pp {
            lg = v.child_ref(p)?;
            v = self.gm.get(lg)?;
            while let Value::Ref(g) = v {
                lg = g;
                v = self.gm.get(g)?;
            }
        }
        Some((lg.clone_weak(), v))
    }

    pub fn get_or_make_child(
        &mut self,
        v: &mut Value,
        p: &ProtoNode,
    ) -> Result<GenData, ActionError> {
        match v {
            Value::Map(m) => match m.get(&p) {
                Some(gd) => Ok(gd.clone_weak()),

                None => Ok(self.gm.push(Value::map())),
            },
            //Value::List(l)=>
            //Ref::
            _ => Err(ActionError::new(
                "Cannot give child to Non Map or List Value",
            )),
        }
    }

    pub fn set(&mut self, p: &Proto, v: Value) -> Result<(), ActionError> {
        let b = self
            .select_base(p)
            .ok_or(ActionError::new("No base"))?
            .clone_weak();
        self.set_from(b, p.pp(), v)
    }

    pub fn set_from(
        &mut self,
        mut c_gd: GenData,
        mut pp: ProtoP,
        nval: Value,
    ) -> Result<(), ActionError> {
        if pp.remaining() == 0 {
            return Err(ActionError::new("empty path"));
        }
        while pp.remaining() > 1 {
            let p = pp.next().unwrap();
            let n_gd = self.gm.push(Value::map());
            let v = match self.gm.get_mut(&c_gd) {
                Some(v) => v,
                None => {
                    self.gm.drop_ref(n_gd);
                    return Err(ActionError::new("no base or deeper problem"))?;
                }
            };
            let n_drop = n_gd.clone_ig();
            c_gd = match v.try_give_child(p, n_gd) {
                Ok((true, g)) => g,
                Ok((false, g)) => {
                    self.gm.drop_ref(n_drop);
                    g
                }
                Err(e) => {
                    self.gm.drop_ref(n_drop);
                    return Err(e);
                }
            };
        }
        let val_ref = self.gm.push(nval);
        let v = self
            .gm
            .get_mut(&c_gd)
            .ok_or(ActionError::new("no base or deeper problem"))?;
        match v.give_child(pp.next().unwrap().clone(), val_ref.clone_weak()) {
            Ok(Some(gdrop)) => {
                self.gm.drop_ref(gdrop);
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(e) => {
                self.gm.drop_ref(val_ref);
                Err(e)
            }
        }
    }

    pub fn call_expr(&mut self, ex: Expr) -> Result<Value, ActionError> {
        self.on_wrap(|sc| ex.eval(sc))
    }

    pub fn for_each<T, IT>(
        &mut self,
        _it: IT,
        _fold: Option<Value>,
        _func: Value,
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
    ) -> Result<Value, ActionError> {
        self.on_wrap(|sc| {
            for p in 0..params.len() {
                if pnames.len() > p {
                    //TODO Set parameters
                }
            }
            sc.do_actions(actions).map(|(_, v)| v)
        })
    }

    pub fn colon_select(&mut self, p: &Proto) -> Result<Value, ActionError> {
        let r = self
            .get_ref(&p)
            .ok_or(ActionError::new("Could not set path to non path"))?;
        let rc = r.clone_weak();
        self.gm.inc_rc(&rc);
        let nb = Base {
            gd: rc,
            swap_off: true,
        };
        let blast = self.bases.len() - 1;
        match self.bases[blast].swap_off {
            true => self.bases[blast] = nb,
            false => self.bases.push(nb),
        }
        Ok(Value::Null)
    }

    //return bool is for is return
    pub fn do_actions(&mut self, actions: &[Action]) -> AcResult {
        let mut last_res = Value::Null;
        for a in actions {
            let r = self.do_action(a)?;
            if let Action::Resolve(_) = a {
                return Ok((AcReturn::No, r));
            }
            if let Action::Return(_) = a {
                return Ok((AcReturn::Func, r));
            }
            last_res = r;
        }
        Ok((AcReturn::No, last_res))
    }
    pub fn do_action(&mut self, a: &Action) -> Result<Value, ActionError> {
        match a {
            Action::Set(p_ex, v_ex) => {
                let p = p_ex.eval_path(self)?;
                let v = v_ex.eval(self)?;
                self.set(&p, v).map(|_| Value::Null)
            }
            Action::Resolve(p_ex) | Action::Return(p_ex) => {
                let v = p_ex.eval(self)?;
                //println!(" - {}", v.print(0, &self.gm));
                Ok(v)
            }
            Action::Select(p_ex) => {
                let p = p_ex.eval_path(self)?;
                self.colon_select(&p)
            }
            Action::SetSelect(p_ex, v_ex) => {
                let p = p_ex.eval_path(self)?;
                let v = v_ex.eval(self)?;
                self.set(&p, v)?;
                self.colon_select(&p)
            }
            Action::AddItem(num, id) => {
                let b = self.bases.last().expect("Bases should never be empty");
                let gdo = match self.gm.get(&b.gd) {
                    Some(Value::Map(m)) => match m.get(&ProtoNode::str(id)) {
                        Some(gd) => Some(gd.clone_ig()),
                        None => None,
                    },
                    _ => return Err(ActionError::new("cannot add like this to non map")),
                };
                match gdo {
                    Some(gd) => match self.gm.get_mut(&gd) {
                        Some(Value::Num(ref mut n)) => *n += num,
                        Some(_) | None => {
                            return Err(ActionError::new("Can only add to number values"))
                        }
                    },
                    None => {
                        let gd = self.gm.push(Value::Num(*num));
                        match self.gm.get_mut(&b.gd) {
                            Some(Value::Map(m)) => {
                                m.insert(ProtoNode::str(id), gd);
                            }
                            _ => {
                                return Err(ActionError::new(
                                    "Map  dropped between previous checks",
                                ))
                            }
                        }
                    }
                }
                Ok(Value::Null)
            }
            Action::OpSet(op, lf, rt) => {
                let p = lf.eval_path(self)?;
                let v = op.eval(lf, rt, self)?;
                self.set(&p, v).map(|_| Value::Null)
            } //_ => unimplemented!(),
        }
        /*Select(Expr),
        OpSet(Op, Expr, Expr),
        AddItem(isize, String),
        RemItem(isize, String),
        Return(Expr),
        */
    }
}
