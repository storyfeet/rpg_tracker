use crate::action::Action;
use crate::parse::LineAction;
use crate::proto::Proto;
use crate::value::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct DnData {
    call_stack:Vec<Proto>,
    pub data: Value,
}


impl DnData {
    pub fn new() -> Self {
        DnData {
            call_stack:Vec::new(),
            data: Value::Tree(BTreeMap::new()),
        }
    }


    pub fn do_action(&mut self, a: LineAction) -> Result<Value, failure::Error> {
        match a.action.clone() {
            Action::Select(proto) => {
                if proto.dot {
                    self.extend_curr(proto.pp());
                } else {
                    self.curr = proto.clone();
                }
                match self.data.get_path(self.curr.pp()) {
                    Some(_) => {}
                    _ => {
                        self.data
                            .set_at_path(self.curr.pp(), Value::tree())
                            .map_err(|_| a.err("count not Create object for selct"))?;
                    }
                }
            }
            Action::Set(proto, v) => {
                let pp = self.curr.extend_if_dot(&proto);
                self.data
                    .set_at_path(pp.pp(), v)
                    .map_err(|_| a.err("Could not Set"))?;
            }
            Action::Add(proto, v) => {
                let pp = self.curr.extend_if_dot(&proto);
                match self.data.get_path(pp.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_add(v)?;
                        self.data
                            .set_at_path(pp.pp(), nv)
                            .map_err(|_| a.err("Could not Add"))?;
                    }
                    None => {
                        self.data
                            .set_at_path(pp.pp(), v)
                            .map_err(|_| a.err("Coult not add"))?;
                    }
                }
            }
            Action::Sub(proto, v) => {
                let pp = self.curr.extend_if_dot(&proto);
                match self.data.get_path(pp.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_sub(v)?;
                        self.data
                            .set_at_path(pp.pp(), nv)
                            .map_err(|_| a.err("Could not Add"))?;
                    }
                    None => {
                        self.data
                            .set_at_path(pp.pp(), v)
                            .map_err(|_| a.err("Coult not add"))?;
                    }
                }
            }
            Action::Expr(e) => return Ok(e.eval(self)?),
            Action::CallFunc(proto,params)=>{
            }
        };
        Ok(Value::num(0))
    }
}
