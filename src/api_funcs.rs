use crate::error::ActionError;
use crate::expr::Expr;
use crate::scope::Scope;
use crate::value::Value;

pub fn d(sc: &mut Scope, params: &[Expr]) -> Result<Option<Value>, ActionError> {
    let p1 = params.get(0).ok_or(ActionError::new("d needs num sides"))?;
    let v = p1.eval(sc)?;
    match v {
        Value::Num(n) => Ok(Some(Value::Num((rand::random::<i32>().abs() % n) + 1))),
        _ => Err(ActionError::new("d needs num sides")),
    }
}

pub fn load(sc: &mut Scope, params: &[Expr]) -> Result<Option<Value>, ActionError> {
    //param order fname, target
    let p1 = params.get(0).ok_or(ActionError::new("d needs num sides"))?;
    let fv = match p1.eval(sc)? {
        Value::Str(s) => s,
        _ => return Err(ActionError::new("fname should be string")),
    };
    match params.get(1) {
        Some(ex) => {
            if let Value::Proto(p) = ex.eval(sc)? {
                let new_sc = Scope::from_file(fv)?;
                sc.set_pp(p.pp(), new_sc.eat_data())?;
                Ok(None)
            } else {
                Err(ActionError::new("target should be proto"))
            }
        }
        None => {
            sc.run_file(fv)?;
            Ok(None)
        }
    }
}
