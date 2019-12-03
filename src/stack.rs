use crate::proto::Proto;
use crate::value::Value;

#[derive(Debug)]
pub struct StackItem {
    base: Proto,
    curr: Option<Proto>,
    pub vars: Value,
}

impl StackItem {
    pub fn new(p: Proto)->Self {
        StackItem {
            base: p,
            curr: None,
            vars: Value::tree(),
        }
    }

    pub fn set_curr(&mut self, p: Proto) {
        self.curr = Some(p)
    }

    pub fn in_context(&self,p:&Proto)->Proto{
        match p.dots{
            0=> p.clone(),
            1=> self.curr.as_ref().unwrap_or(&self.base).extend_new(p.pp()),
            _=> self.base.extend_new(p.pp()),
        }
    }


}
