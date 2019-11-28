use crate::error::ActionError;
use crate::parse::Action;
use crate::value::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct DnData {
    curr: Option<String>,
    data: BTreeMap<String, Value>,
}

impl DnData {
    pub fn new() -> Self {
        DnData {
            curr: None,
            data: BTreeMap::new(),
        }
    }

    pub fn do_action(&mut self, a: Action) -> Result<(), ActionError> {
        match a {
            Action::SetItem(name) => {
                if self.data.get(&name) == None {
                    let ni = DnDItem::new(name.clone());
                    self.data.insert(name.clone(), ni);
                }
                self.curr = Some(name);
            }
            Action::SetStat(n, v) => {
                self.current_item()
                    .ok_or(ActionError::new("no item"))?
                    .set_data(n, v);
            }
            Action::AddStat(n, v) => {
                self.current_item()
                    .ok_or(ActionError::new("no item"))?
                    .add_data(n, v);
            }
            Action::SubStat(n, v) => {
                self.current_item()
                    .ok_or(ActionError::new("no item"))?
                    .sub_data(n, v);
            }
            Action::GainItem(i, n) => self
                .current_item()
                .ok_or(ActionError::new("no item"))?
                .gain_item(i, n),
        };
        Ok(())
    }
}
