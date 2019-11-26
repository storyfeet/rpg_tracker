use crate::error::ActionError;
use crate::expr::Expr;
use crate::parse::Action;
use std::collections::BTreeMap;
use std::ops::Add;

#[derive(Debug, PartialEq)]
pub struct DnDItem {
    name: String,
    data: BTreeMap<String, Value>,
    items: BTreeMap<String, i32>,
}

impl DnDItem {
    pub fn new(name: String) -> Self {
        DnDItem {
            name: name,
            data: BTreeMap::new(),
            items: BTreeMap::new(),
        }
    }

    pub fn set_data(&mut self, name: String, v: Value, rel: bool) {
        if !rel {
            self.data.insert(name, v);
            return;
        }
        let s = self.data.remove(&name);
        match s {
            None => self.data.insert(name, v),
            Some(old) => self.data.insert(name, old + v),
        };
    }
}

#[derive(Debug)]
pub struct DnData {
    curr: Option<String>,
    ctype: String,
    data: BTreeMap<String, DnDItem>,
}

impl DnData {
    pub fn new() -> Self {
        DnData {
            curr: None,
            ctype: "Player".to_string(),
            data: BTreeMap::new(),
        }
    }

    pub fn current_item(&mut self) -> Option<&mut DnDItem> {
        self.data.get_mut(self.curr.as_ref()?)
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
                    .set_data(n, v.into(), false);
            }
            Action::AddStat(n, v) => {
                self.current_item()
                    .ok_or(ActionError::new("no item"))?
                    .set_data(n, v.into(), true);
            }
            _ => {}
        };
        Ok(())
    }
}
