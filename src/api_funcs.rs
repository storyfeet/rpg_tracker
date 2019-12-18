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

pub fn if_expr(sc: &mut Scope, params: &[Expr]) -> Result<Option<Value>, ActionError> {
    if params.len() < 3 {
        return Err(ActionError::new("if requires 3 params"));
    }
    match params[0].eval(sc) {
        Ok(Value::Bool(true)) => params[1].eval(sc).map(|v| Some(v)),
        Ok(Value::Num(n)) if n > 0 => params[1].eval(sc).map(|v| Some(v)),
        _ => params[2].eval(sc).map(|v| Some(v)),
    }
}

/// final function should take (k,v) as params
pub fn for_each(sc: &mut Scope, params: &[Expr]) -> Result<Option<Value>, ActionError> {
    if params.len() <= 1 {
        return Err(ActionError::new("Foreach requires at least 2 params"));
    }
    if params.len() == 2 {
        println!("2 params");
        if let Expr::Val(ref func) = params[1] {
            match params[0] {
                Expr::Val(Value::List(ref l)) => {
                    return sc.for_each(l.clone().into_iter().enumerate(), func.clone())
                }
                _ => return Err(ActionError::new("Must be list for iterator right now")),
            }
        }
    }

    Ok(None)
}
