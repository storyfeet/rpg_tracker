use crate::error::ActionError;
use crate::scope::Scope;
use crate::value::Value;

pub fn run_api_expr(fname:&str,scope:&Scope,params:&[Value])->Option<Result<Option<Value>,ActionError>>{
    scope.on_wrap(|wrap|{run_api_func(fname,wrap,params)})
}

pub fn run_api_func(fname:&str,scope:&mut Scope,params:&[Value])->Option<Result<Option<Value>,ActionError>>{
    Some(match fname{
        "d" => d(scope,params),
        "foreach" => for_each(scope,&params),
        "fold" => fold(scope,&params),
        "load" => load(scope,&params),
        "if" => if_expr(scope,&params),
        "link" => link(scope,&params),
        _ => {return None}
    })
}

pub fn d(_sc: &mut Scope, params: &[Value]) -> Result<Option<Value>, ActionError> {
    let mut res = 0;
    for p in params {
        match p {
            Value::Num(n) => res += rand::random::<i32>().abs() % n + 1,
            _ => return Err(ActionError::new("d needs num sides")),
        }
    }
    Ok(Some(Value::Num(res)))
}

pub fn link(_sc: &mut Scope, params: &[Value]) -> Result<Option<Value>, ActionError> {
    match params.get(0) {
        Some(Value::Proto(p)) => Ok(Some(Value::Proto(p.with_deref(1)))),
        _ => Err(ActionError::new("can only link on protos")),
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
            sc.set(p, new_sc.eat_data())?;
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

pub fn fold(sc: &mut Scope, params: &[Value]) -> Result<Option<Value>, ActionError> {
    match params.len() {
        n if n < 3 => Err(ActionError::new(
            "Fold requires 3 params : foldvar,iterble,func",
        )),
        _ => fold_each(sc, params.get(0).map(|n| n.clone()), &params[1..]),
    }
}

pub fn for_each(sc: &mut Scope, params: &[Value]) -> Result<Option<Value>, ActionError> {
    fold_each(sc, None, params)
}

/// final function should take (k,v) as params
fn fold_each(
    sc: &mut Scope,
    fold: Option<Value>,
    params: &[Value],
) -> Result<Option<Value>, ActionError> {
    if params.len() <= 1 {
        return Err(ActionError::new("FoldEach requires at least 2 params"));
    }
    if params.len() == 2 {
        match params[0] {
            Value::List(ref l) => {
                return sc.for_each(l.clone().into_iter().enumerate(), fold, params[1].clone())
            }
            Value::Num(n) => {
                return sc.for_each(
                    (0..n).map(|x| Value::Num(x)).enumerate(),
                    fold,
                    params[1].clone(),
                )
            }
            _ => return Err(ActionError::new("Must be list for iterator right now")),
        }
    }

    Ok(None)
}
