use crate::error::ActionError;
use crate::scope::Scope;
use crate::value::Value;

pub fn d(_sc: &mut Scope, params: &[Value]) -> Result<Option<Value>, ActionError> {
    match params[0] {
        Value::Num(n) => Ok(Some(Value::Num((rand::random::<i32>().abs() % n) + 1))),
        _ => Err(ActionError::new("d needs num sides")),
    }
}

pub fn load(sc: &mut Scope, params: &[Value]) -> Result<Option<Value>, ActionError> {
    //param order fname, target
    let p1 = params.get(0).ok_or(ActionError::new("d needs num sides"))?;
    let fv = match p1 {
        Value::Str(s) => s,
        _ => return Err(ActionError::new("filename should be string")),
    };
    match params.get(1) {
        Some(Value::Proto(p)) => {
            let new_sc = Scope::from_file(fv)?;
            sc.set_pp(p.pp(), new_sc.eat_data())?;
            Ok(None)
        }
        Some(_) => Err(ActionError::new("target should be proto")),

        None => {
            sc.run_file(fv)?;
            Ok(None)
        }
    }
}

pub fn if_expr(_sc: &mut Scope, params: &[Value]) -> Result<Option<Value>, ActionError> {
    if params.len() < 3 {
        return Err(ActionError::new("if requires 3 params"));
    }
    match params[0] {
        Value::Bool(true) => Ok(Some(params[1].clone())),
        Value::Num(n) if n > 0 => Ok(Some(params[1].clone())),
        _ => Ok(Some(params[2].clone())),
    }
}

/// final function should take (k,v) as params
pub fn for_each(sc: &mut Scope, params: &[Value]) -> Result<Option<Value>, ActionError> {
    if params.len() <= 1 {
        return Err(ActionError::new("Foreach requires at least 2 params"));
    }
    if params.len() == 2 {
        match params[0] {
            Value::List(ref l) => {
                return sc.for_each(l.clone().into_iter().enumerate(), params[1].clone())
            }
            _ => return Err(ActionError::new("Must be list for iterator right now")),
        }
    }

    Ok(None)
}
