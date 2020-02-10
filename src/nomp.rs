use nom::branch::alt;
use nom::bytes::complete::{escaped, is_not, tag, take};
use nom::character::complete::*;
use nom::combinator::{map, peek, value};
use nom::multi::separated_list;
use nom::sequence::*;
use nom::{error::ErrorKind, IResult};

use std::collections::BTreeMap;

use crate::expr::{Expr, Op};
use crate::value::Value;

pub fn n_ident(s: &str) -> IResult<&str, &str> {
    preceded(peek(alpha1), alphanumeric1)(s)
}

pub fn n_bool(s: &str) -> IResult<&str, bool> {
    alt((
        value(true, alt((tag("true"), tag("TRUE"), tag("True")))),
        value(false, alt((tag("false"), tag("FALSE"), tag("False")))),
    ))(s)
}

pub fn n_qoth(s: &str) -> IResult<&str, &str> {
    delimited(
        tag("\""),
        escaped(is_not("\"\\"), '\\', take(1u8)),
        tag("\""),
    )(s)
}

pub fn n_num(s: &str) -> IResult<&str, i32> {
    match digit1(s) {
        Ok((i, r)) => match r.parse::<i32>() {
            Ok(pr) => Ok((i, pr)),
            _ => Err(nom::Err::Error((s, ErrorKind::TooLarge))),
        },
        Err(e) => Err(e),
    }
}

pub fn d_expr_def(s: &str) -> IResult<&str, Value> {
    map(
        tuple((w_tag("expr"), delimited(w_tag("("), r_expr, w_tag(")")))),
        |(_, e)| Value::ExprDef(Box::new(e)),
    )(s)
}

pub fn d_func_def(s: &str) -> IResult<&str, Value> {
    //TODO include actual function actions
    let sep = delimited(space0, tag(","), space0);
    let id_list = separated_list(sep, n_ident);
    let params = delimited(tag("("), id_list, tag(")"));
    let res = tuple((tag("func"), params));
    res(s).map(|(n, (_, pars))| {
        (
            n,
            Value::FuncDef(
                pars.iter().fold(Vec::new(), |mut v, s| {
                    v.push(s.to_string());
                    v
                }),
                Vec::new()
            ),
        )
    })
}

pub fn d_value(s: &str) -> IResult<&str, Value> {
    alt((
        map(n_bool, |v| Value::Bool(v)),
        map(n_qoth, |v| Value::Str(v.to_string())), //TODO escape
        map(n_num, |v| Value::Num(v)),
        d_func_def,
        d_expr_def,
    ))(s)
}


/*pub fn wsp<'a,F,O,E>(f:F)->impl Fn(&'a str)->IResult<&'a str,O,E>
    where //I:Clone+nom::InputLength+nom::InputIter+nom::InputTake,//+nom::UnspecializedInput,
          //<I as nom::InputIter>::Item: nom::AsChar+Clone,
          F:Fn(I)->IResult<I,O,E>,
          E:nom::error::ParseError<&'a str>,
{
    //wrap(space0,f)
    move |s:&str| wrap(space0,f)(s)
}*/

/*pub fn w_tag<'a, T:'a,I:'a,O,E>(t:T) -> impl Fn(I) -> IResult<I, O,E>
    where I:Clone+nom::InputLength+nom::InputIter+nom::InputTake + nom::Compare<T>+nom::UnspecializedInput,
          T:nom::InputLength + Clone,
          <I as nom::InputIter>::Item: nom::AsChar+Clone,
{
    move |s:I|wrap(space0,tag(t))(s)
    //move |s| map(tuple((space0, tag(t), space0)), |_| ())(s)
}
*/

pub fn w_tag<'a>(t:&'static str)->impl Fn(&str)-> IResult<&str,()>{
    move |s:&str|map(wrap(space0,tag(t)),|_|())(s)
}

pub fn e_neg(s: &str) -> IResult<&str, Expr> {
    map(tuple((wrap(space0,tag("-")), r_expr)), |(_, e)| Expr::Neg(Box::new(e)))(s)
}
pub fn e_dot(s: &str) -> IResult<&str, Expr> {
    map(tuple((wrap(space0,tag(".")), r_expr)), |(_, e)| Expr::Dot(Box::new(e)))(s)
}

pub fn e_bracket(s: &str) -> IResult<&str, Expr> {
    map(delimited(wrap(space0,tag("(")), r_expr, wrap(space0,tag(")"))), |e| {
        Expr::Bracket(Box::new(e))
    })(s)
}

pub fn e_map(s: &str) -> IResult<&str, Expr> {

    let colons = tuple((n_ident, wrap(space0,tag(":")), r_expr));
    map(
        delimited(w_tag("["), separated_list(w_tag(","), colons), w_tag("]")),
        |l| {
            let mut res = BTreeMap::new();
            for (n, _, v) in l {
                res.insert(n.to_string(), v);
            }
            Expr::Map(res)
        },
    )(s)
}

pub fn e_list(s: &str) -> IResult<&str, Expr> {
    map(
        delimited(wrap(space0,tag("[")), separated_list(wrap(space0,tag(",")), r_expr), wrap(space0,tag("]"))),
        |l| Expr::List(l),
    )(s)
}

pub fn wrap<W,F,I,O,O2,E>(fw:W,fm:F)->impl Fn(I)->IResult<I,O,E>
    where W:Fn(I)->IResult<I,O2,E>,
          F:Fn(I)->IResult<I,O,E>,
          E:nom::error::ParseError<I>,
          I:Clone,
{
    map(tuple((fw,fm,fw)),|(_,r,_)|r)
}



pub fn e_op(s:&str)->IResult<&str,Op>{
    wrap(space0,
        alt((
            map(tag("=="),|_|Op::Equal),
            map(one_of(".+-*/><"),|c| Op::from_char(c)),
        ))
    )(s)

}

//right expr try to parse biggest thing first
pub fn r_expr(s: &str) -> IResult<&str, Expr> {
    alt((
        map(
            tuple((
                l_expr,
                e_op,
                r_expr,
            )),
            |(l, o, r)| r.add_left(l, o),
        ),
        l_expr,
    ))(s)
}

//left expr pass as soon as possible
pub fn l_expr(s: &str) -> IResult<&str, Expr> {
    alt((
        e_neg,
        e_dot,
        e_bracket,
        e_list,
        e_map,
        map(d_value, |v| Expr::Val(v)),
    ))(s)
}

#[cfg(test)]
mod nom_test {
    use super::*;
    use crate::scope::Scope;
    #[test]
    fn test_nom_ident() {
        assert_eq!(n_ident("hello"), Ok(("", "hello")));
        assert_eq!(n_ident("h2ello "), Ok((" ", "h2ello")));
        assert!(n_ident("  hello").is_err());
        assert!(n_ident("1hello").is_err());
        assert!(n_ident(" abc").is_err());
        assert!(n_ident("123").is_err());
    }

    #[test]
    fn test_qoth() {
        assert_eq!(n_qoth(r#""fish"sr"#), Ok(("sr", "fish")));
        assert_eq!(n_qoth(r#""fish\"sr""#), Ok(("", "fish\\\"sr")));
        assert!(n_qoth(r#""erjkker"#).is_err());
    }

    #[test]
    fn test_nom_value() {
        assert_eq!(d_value("true"), Ok(("", Value::Bool(true))));
        assert_eq!(d_value("True"), Ok(("", Value::Bool(true))));
        assert_eq!(d_value("34hell"), Ok(("hell", Value::Num(34))));
    }

    #[test]
    fn test_func_def() {
        assert_eq!(
            d_func_def("func(fish,green,pink)"),
            Ok((
                "",
                Value::FuncDef(
                    vec!["fish".to_string(), "green".to_string(), "pink".to_string()],
                    Vec::new()
                )
            ))
        );

        assert!(d_func_def("fn(fish green pink)").is_err());
    }

    fn part_eval(s: &str) -> Value {
        let sc = Scope::new();
        let e = r_expr(s).unwrap().1;
        e.eval(&sc).unwrap()
    }
    #[test]
    fn test_eval_expr() {
        assert_eq!(part_eval("3+4"), Value::Num(7));
        assert_eq!(part_eval("3+4*5"), Value::Num(23));
        assert_eq!(part_eval("5*3+4"), Value::Num(19));
        assert_eq!(part_eval("3*(4+5)"), Value::Num(27));
        assert_eq!(part_eval("3 * ( 4 + 5 ) "), Value::Num(27));
    }
}
