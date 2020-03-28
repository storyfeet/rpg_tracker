use gobble::combi::*;
use gobble::err::*;
use gobble::ptrait::*;
use gobble::reader::*;
use std::str::FromStr;

use crate::expr::{Expr, Op};
use crate::value::Value;

fn parse_value<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Value> {
    let (r, _) = ws(0).parse(i)?;
    if let Ok((r, v)) = tag("true").or(tag("false")).parse(&r) {
        return Ok((r, Value::Bool(v == "true")));
    }
    if let Ok((r, v)) = read_f::<_, _, String>(is_num, 1).parse(&r) {
        return Ok((r, Value::Num(i32::from_str(&v).unwrap())));
    }
    if let Ok((r, v)) = tag("\"").ig_then(esc('"', '\\').e_map('t', '\t')).parse(&r) {
        return Ok((r, Value::Str(v)));
    }
    if let Ok((r, (mut a, b))) = read_f::<_, _, String>(is_alpha, 1)
        .then(maybe(read_f::<_, _, String>(is_alpha_num, 0)))
        .parse(&r)
    {
        if let Some(bs) = b {
            a.push_str(&bs);
        }
        return Ok((r, Value::Ident(a)));
    }

    return Err(ParseError::new("No value parseable", 0));
}

fn parse_op<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Op> {
    let parser = ws(0)
        .ig_then(
            tag("+")
                .or(tag("-"))
                .or(tag("*"))
                .or(tag("/"))
                .or(tag("=="))
                .or(tag("."))
                .or(tag(">"))
                .or(tag("<"))
                .or(tag("/")),
        )
        .then_ig(ws(0));
    let (ri, c) = parser.parse(i)?;
    let rop = Op::from_str(c)?;
    Ok((ri, rop))
}

fn parse_expr_l<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Expr> {
    if let Ok((ir, e)) = parse_value.parse(i) {
        return Ok((ir, Expr::Val(e)));
    }
    if let Ok((ir, e)) = tag("-").ig_then(parse_expr_l).parse(i) {
        return Ok((ir, Expr::Neg(Box::new(e))));
    }
    if let Ok((ir, e)) = tag("(").ig_then(parse_expr).then_ig(tag(")")).parse(i) {
        return Ok((ir, Expr::Bracket(Box::new(e))));
    }
    Err(ParseError::new("Expr Left fail", 0))
}

pub fn parse_expr<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Expr> {
    let (ir, l) = parse_expr_l.parse(i)?;
    if let Ok((ir, (op, v2))) = parse_op.then(parse_expr).parse(&ir) {
        return Ok((ir, v2.add_left(l, op)));
    }
    Ok((ir, l))
}
