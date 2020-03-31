use crate::error::ActionError;
use crate::value::Value;

#[derive(Debug)]
pub struct GenData {
    pos: usize,
    gen: u64,
    strong: bool, //Not part of eq
}

impl PartialEq for GenData {
    fn eq(&self, rhs: &Self) -> bool {
        self.pos == rhs.pos && self.gen == rhs.gen
    }
}

impl GenData {
    pub fn clone_strong(&self, gm: &mut GenManager) -> Option<GenData> {
        if gm.inc_rc(self) {
            Some(GenData {
                pos: self.pos,
                gen: self.gen,
                strong: true,
            })
        } else {
            None
        }
    }

    pub fn clone_weak(&self) -> GenData {
        GenData {
            pos: self.pos,
            gen: self.gen,
            strong: false,
        }
    }
    pub fn drop_weak(self) -> Result<(), ActionError> {
        if self.strong {
            Err(ActionError::new(
                "attempted to drop strong ref as though weak",
            ))
        } else {
            Ok(())
        }
    }
    pub fn drop_rc(self, gm: &mut GenManager) {
        if self.strong {
            gm.dec_rc(self);
        }
    }
}

pub struct StoreItem {
    val: Option<Value>,
    gen: u64,
    rc: u64,
}

#[derive(Debug)]
pub struct GenManager {
    drops: Vec<usize>,
    items: Vec<StoreItem>,
}

impl GenManager {
    pub fn new() -> Self {
        GenManager {
            drops: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn get<'a>(&'a self, gd: &GenData) -> Option<&'a Value> {
        let rs = self.items.get(gd.pos)?;
        if gd.gen != rs.gen {
            return None;
        }
        rs.val.as_ref()
    }

    pub fn get_mut<'a>(&'a mut self, gd: &GenData) -> Option<&'a Value> {
        let mut rs = self.items.get(gd.pos)?;
        if gd.gen != rs.gen {
            return None;
        }
        rs.val.as_ref()
    }

    pub fn push(&mut self, v: Value) -> GenData {
        if let Some(loc) = self.drops.pop() {
            let ea = &mut self.items[loc];
            ea.val = Some(v);
            ea.gen += 1;
            ea.rc = 1;
            return GenData {
                pos: loc,
                gen: ea.gen,
                strong: true,
            };
        }
        self.items.push(StoreItem {
            val: Some(v),
            gen: 0,
            rc: 1,
        });
        return GenData {
            gen: 0,
            pos: self.items.len() - 1,
            strong: true,
        };
    }

    pub fn dec_rc(&mut self, g: GenData) {
        if let Some(ea) = self.items.get_mut(g.pos) {
            if ea.gen == g.gen && ea.rc > 0 {
                ea.rc -= 1;
                if ea.rc == 0 {
                    if let Some(v) = ea.val.take() {
                        for vd in v.gen_drop() {
                            self.dec_rc(vd);
                        }
                        self.drops.push(g.pos);
                    }
                }
            }
        }
    }

    pub fn inc_rc(&mut self, g: &GenData) -> bool {
        if let Some(ea) = self.items.get_mut(g.pos) {
            if ea.gen == g.gen {
                ea.rc += 1;
                return true;
            }
        }
        false
    }
}
