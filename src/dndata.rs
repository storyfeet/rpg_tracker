use crate::action::Action;
use crate::error::ActionError;
use crate::parse::LineAction;
use crate::proto::ProtoStack;
use crate::value::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct DnData {
    stack: ProtoStack,
    pub data: Value,
}

impl DnData {
    pub fn new() -> Self {
        DnData {
            stack: ProtoStack::new(),
            data: Value::Tree(BTreeMap::new()),
        }
    }

    pub fn do_action(&mut self, a: LineAction) -> Result<Value, failure::Error> {
        fn err(s: &str) -> ActionError {
            ActionError::new(s)
        };
        match a.action {
            Action::Select(proto) => {
                let np = self.stack.in_context(proto);
                match self.data.get_path(np.pp()) {
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
                self.data
                    .set_at_path(np.pp(), v)
                    .map_err(|_| err("Could not Set"))?;
            }
            Action::Add(proto, v) => {
                let np = self.stack.in_context(proto);
                match self.data.get_path(np.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_add(v)?;
                        self.data
                            .set_at_path(np.pp(), nv)
                            .map_err(|_| err("Could not Add"))?;
                    }
                    None => {
                        self.data
                            .set_at_path(np.pp(), v)
                            .map_err(|_| err("Coult not add"))?;
                    }
                }
            }
            Action::Sub(proto, v) => {
                let np = self.stack.in_context(proto);
                match self.data.get_path(np.pp()) {
                    Some(ov) => {
                        let nv = ov.clone().try_sub(v)?;
                        self.data
                            .set_at_path(np.pp(), nv)
                            .map_err(|_| err("Could not Add"))?;
                    }
                    None => {
                        self.data
                            .set_at_path(np.pp(), v)
                            .map_err(|_|err("Coult not add"))?;
                    }
                }
            }
            Action::Expr(e) => return Ok(e.eval(self)?),
            Action::CallFunc(proto, _params) => {
                //TODO work out how to pass params
                let np = self.stack.in_context(proto);
                self.stack.save();
                if let Some(Value::FuncDef(_pnames, _actions)) = self.data.get_path(np.pp()) {
                    //TODO
                }

                self.stack.restore();
            }
        };
        Ok(Value::num(0))
    }
}
