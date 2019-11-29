use crate::error::ActionError;
use crate::parse::Action;
use crate::proto::Proto;
use crate::value::{GotPath, Value};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct DnData {
    curr: Proto,
    data: Value,
}

impl DnData {
    pub fn new() -> Self {
        DnData {
            curr: Proto::empty(),
            data: Value::Tree(BTreeMap::new()),
        }
    }

    pub fn do_action(&mut self, a: Action) -> Result<(), ActionError> {
        match a {
            Action::SetItem(name) => {
                self.curr = Proto::new(&name);
                match self.data.get_path(self.curr.pp()) {
                    GotPath::Val(_) => {}
                    _ => {
                        self.data.set_at_path(self.curr.pp(), Value::tree());
                    }
                }
            }
            Action::SetStat(n, v) => {}
            Action::AddStat(n, v) => {}
            Action::SubStat(n, v) => {}
            Action::GainItem(i, n) => {}
        };
        Ok(())
    }
}
