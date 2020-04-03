use crate::action::Action;
use crate::ecs_ish::{GenData, GenManager};
use crate::error::ActionError;
use crate::expr::Expr;
use crate::proto::{Proto, ProtoNode, ProtoP};
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
        let mut ss = LCChars::str(s);
        let ac = crate::nomp::action();
        loop {
            let (ns, a) = ac.parse(&ss)?;
            println!("Action = {:?}", a);
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
            last
        };
        self.bases.get(n).map(|v| &v.gd)
    }

    pub fn get<'a>(&'a self, p: &Proto) -> Option<&'a Value> {
        let b = self.select_base(p)?.clone_ignore_gm();
        return self.get_from(&b, p.pp());
    }

    pub fn get_from<'a>(&'a self, base: &GenData, pp: ProtoP) -> Option<&'a Value> {
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

    pub fn get_or_make_child(
        &mut self,
        v: &mut Value,
        p: &ProtoNode,
    ) -> Result<GenData, ActionError> {
        match v {
            Value::Map(m) => match m.get(&p) {
                Some(v) => Ok(v.clone(&mut self.gm)),
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
            .clone_ignore_gm();
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
            let v = self
                .gm
                .get_mut(&c_gd)
                .ok_or(ActionError::new("no base or deeper problem"))?;
            c_gd = match v.try_give_child(p, &n_gd) {
                Some((true, g)) => g,
                Some((false, g)) => {
                    self.gm.drop_ref(n_gd);
                    g
                }
                None => {
                    self.gm.drop_ref(n_gd);
                    return Err(ActionError::new("Could not give child"));
                }
            };
        }
        let val_ref = self.gm.push(nval);
        let v = self
            .gm
            .get_mut(&c_gd)
            .ok_or(ActionError::new("no base or deeper problem"))?;
        match v.give_child(pp.next().unwrap().clone(), val_ref.clone_ignore_gm()) {
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

    pub fn do_action(&mut self, a: &Action) -> Result<Option<Value>, ActionError> {
        match a {
            Action::NoOp => Ok(None),
            Action::Set(p_ex, v_ex) => {
                let p = p_ex.eval_path(self)?;
                let v = v_ex.eval(self)?;
                self.set(&p, v).map(|_| None)
            }
            Action::Display(p_ex) => {
                let v = p_ex.eval(self)?;
                println!(" - {}", v.print(0, &self.gm));
                Ok(None)
            }
            _ => unimplemented!(),
        }
        /*Select(Expr),
        OpSet(Op, Expr, Expr),
        AddItem(isize, String),
        RemItem(isize, String),
        Return(Expr),
        */
    }
}
