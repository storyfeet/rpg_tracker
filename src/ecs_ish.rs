pub trait GenDrop {
    fn gen_drop(self) -> Vec<GenData>;
}

#[derive(PartialEq)]
pub struct GenData {
    pub pos: usize,
    pub gen: u64,
}

impl GenData {
    pub fn clone_rc<V: GenDrop>(&self, gm: &mut GenManager<V>) -> GenData {
        gm.inc_rc(self);
        GenData {
            pos: self.pos,
            gen: self.gen,
        }
    }
    pub fn drop_rc<V: GenDrop>(self, gm: &mut GenManager<V>) {
        gm.dec_rc(self);
    }
}

pub struct StoreItem<V: GenDrop> {
    val: Option<V>,
    gen: u64,
    rc: u64,
}

pub struct GenManager<V: GenDrop> {
    drops: Vec<usize>,
    items: Vec<StoreItem<V>>,
}

impl<V: GenDrop> GenManager<V> {
    pub fn new() -> Self {
        GenManager {
            drops: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn push(&mut self, v: V) -> GenData {
        if let Some(loc) = self.drops.pop() {
            let ea = &mut self.items[loc];
            ea.val = Some(v);
            ea.gen += 1;
            ea.rc = 1;
            return GenData {
                pos: loc,
                gen: ea.gen,
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

    pub fn inc_rc(&mut self, g: &GenData) {
        if let Some(ea) = self.items.get_mut(g.pos) {
            if ea.gen == g.gen {
                ea.rc += 1
            }
        }
    }
}
