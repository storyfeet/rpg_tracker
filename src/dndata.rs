use crate::parse::{Action, LineAction};
use crate::proto::Proto;
use crate::value::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct DnData {
    curr: Proto,
    data: Value,
}

impl DnData {
    pub fn new() -> Self {
        DnData {
            curr: Proto::empty(false),
            data: Value::Tree(BTreeMap::new()),
        }
    }

    pub fn do_action(&mut self, a: LineAction) -> Result<(), failure::Error> {
        match a.action.clone() {
            Action::Select(proto) => {
                if proto.dot {
                    self.curr.extend(proto.pp());
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
        };
        Ok(())
    }
}
