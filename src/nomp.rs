use gobble::combi::*;
use gobble::err::*;
use gobble::ptrait::*;
use gobble::reader::*;
use std::str::FromStr;

use crate::action::Action;

use crate::expr::{EList, Expr, MapItem, Op};
use crate::value::Value;

pub fn p_action<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Action> {
    pp_action.then_ig(p_break).parse(i)
}
pub fn pp_action<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Action> {
    if let Ok((r2, (opn, id))) = tag("+").ig_then(maybe(p_num)).then(p_ident).parse(i) {
        return Ok((r2, Action::AddItem(opn.unwrap_or(1), id)));
    }
    if let Ok((r2, (opn, id))) = tag("-").ig_then(maybe(p_num)).then(p_ident).parse(i) {
        return Ok((r2, Action::RemItem(opn.unwrap_or(1), id)));
    }
    let (r, ex) = p_expr_l.then_ig(ws(0)).parse(i)?;
    if let Ok((r2, _)) = tag(":").parse(&r) {
        return Ok((r2, Action::Select(ex)));
    }
    if let Ok((r2, (op, ex2))) = p_op.then_ig(tag("=")).then(p_expr).parse(&r) {
        return Ok((r2, Action::OpSet(op, ex, ex2)));
    }
    if let Ok((r2, ex2)) = tag("=").ig_then(p_expr).parse(&r) {
        return Ok((r2, Action::Set(ex, ex2)));
    }
    Ok((r, Action::Display(ex)))
}

fn p_ident<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, String> {
    let (r, (mut a, b)) = read_fs(is_alpha, 1)
        .then(read_fs(is_alpha_num, 0))
        .parse(i)?;
    a.push_str(&b);
    return Ok((r, a));
}
fn p_num<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, i32> {
    let (r, v) = read_fs(is_num, 1).parse(i)?;
    Ok((r, i32::from_str(&v).unwrap()))
}

fn p_break<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, ()> {
    ws(0).then_ig(tag(";").or(tag("\n"))).parse(i)
}

fn p_value<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Value> {
    let (r, _) = ws(0).parse(i)?;
    if let Ok((r, v)) = tag("true").or(tag("false")).parse(&r) {
        return Ok((r, Value::Bool(v == "true")));
    }
    if let Ok((r, v)) = p_num(&r) {
        return Ok((r, Value::Num(v)));
    }
    if let Ok((r, v)) = tag("\"").ig_then(esc('"', '\\').e_map('t', '\t')).parse(&r) {
        return Ok((r, Value::Str(v)));
    }
    if let Ok((r, v)) = p_ident(&r) {
        return Ok((r, Value::Ident(v)));
    }

    return Err(ParseError::new("No value parseable", 0));
}

fn p_op<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Op> {
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

fn p_list<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, EList<Expr>> {
    if let Ok((ir, e)) = p_expr(i) {
        if let Ok((ir2, l)) = tag(",").ig_then(p_list).parse(&ir) {
            return Ok((ir2, EList(Some(Box::new((e, l))))));
        }
    }
    Ok((i.clone(), EList(None)))
}

fn p_map_item<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, MapItem> {
    p_ident
        .then_ig(tag(":"))
        .then(p_expr)
        .parse(i)
        .map(|(r, (k, v))| (r, MapItem { k, v }))
}

fn p_map<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, EList<MapItem>> {
    if let Ok((ir, e)) = p_map_item(i) {
        if let Ok((ir2, l)) = tag(",").ig_then(p_map).parse(&ir) {
            return Ok((ir2, EList(Some(Box::new((e, l))))));
        }
    }
    Ok((i.clone(), EList(None)))
}

fn p_expr_l<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Expr> {
    let (i2, _) = ws(0).parse(i)?;
    if let Ok((ir, e)) = p_value.parse(&i2) {
        return Ok((ir, Expr::Val(e)));
    }
    if let Ok((ir, e)) = tag("-").ig_then(p_expr_l).parse(&i2) {
        return Ok((ir, Expr::Neg(Box::new(e))));
    }
    if let Ok((ir, e)) = tag("$").ig_then(p_expr_l).parse(&i2) {
        return Ok((ir, Expr::Ref(Box::new(e))));
    }
    if let Ok((ir, e)) = tag("(").ig_then(p_expr).then_ig(tag(")")).parse(&i2) {
        return Ok((ir, Expr::Bracket(Box::new(e))));
    }
    if let Ok((ir, l)) = tag("[").ig_then(p_list).then_ig(tag("]")).parse(&i2) {
        return Ok((ir, Expr::List(l)));
    }
    Err(ParseError::new("Expr Left fail", 0))
}

pub fn p_expr<I: Iterator<Item = char> + Clone>(i: &I) -> ParseRes<I, Expr> {
    let (ir, l) = p_expr_l.parse(i)?;
    if let Ok((ir, (op, v2))) = p_op.then(p_expr).parse(&ir) {
        return Ok((ir, v2.add_left(l, op)));
    }
    Ok((ir, l))
}
