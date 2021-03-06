use gobble::*;
use std::str::FromStr;

use crate::action::Action;

use crate::expr::{Expr, MapItem, Op};

pub fn pp_action<'a>(i: &LCChars<'a>) -> ParseRes<'a, Action> {
    let ps = s_tag("+")
        .ig_then(maybe(num()))
        .then(ident())
        .map(|(nop, s)| Action::AddItem(nop.unwrap_or(1), s))
        .or(s_tag("-")
            .ig_then(maybe(num()))
            .then(ident())
            .map(|(nop, s)| Action::AddItem(-nop.unwrap_or(1), s)))
        .or(s_tag("return").ig_then(p_expr).map(|e| Action::Return(e)));
    if let Ok((r, v)) = ps.parse(i) {
        return Ok((r, v));
    }

    let (r, l_ex) = p_expr.parse(i)?;
    if let Ok((r2, _)) = s_tag(":").parse(&r) {
        return Ok((r2, Action::Select(l_ex)));
    }
    if let Ok((r2, (oper, ex2))) = op().then_ig(tag("=")).then(p_expr).parse(&r) {
        return Ok((r2, Action::OpSet(oper, l_ex, ex2)));
    }
    if let Ok((r2, r_ex)) = s_tag("=").ig_then(p_expr).parse(&r) {
        if let Ok((r3, _)) = s_tag(":").parse(&r2) {
            return Ok((r3, Action::SetSelect(l_ex, r_ex)));
        }
        return Ok((r2, Action::Set(l_ex, r_ex)));
    }
    Ok((r, Action::Resolve(l_ex)))
}

fn ident() -> impl Parser<String> {
    ws(0)
        .ig_then(read_fs(is_alpha, 1))
        .then(read_fs(is_alpha_num, 0))
        .map(|(mut a, b)| {
            a.push_str(&b);
            a
        })
}
fn num() -> impl Parser<isize> {
    ws(0)
        .ig_then(read_fs(is_num, 1))
        .try_map(|ns| isize::from_str(&ns).map_err(|_| ECode::SMess("Not a Num")))
}

pub fn l_break() -> impl Parser<()> {
    ws(0).then_ig(tag(";").or(tag("\n")))
}

fn op() -> impl Parser<Op> {
    ws(0)
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
        .try_map(|c| Op::from_str(c))
}

fn list() -> impl Parser<Expr> {
    s_tag("[")
        .ig_then(sep(p_expr, s_tag(","), false))
        .then_ig(s_tag("]"))
        .map(|l| Expr::List(l))
}

fn map_item() -> impl Parser<MapItem> {
    ident()
        .then_ig(s_tag(":"))
        .then(p_expr)
        .map(|(k, v)| MapItem { k, v })
}

fn map() -> impl Parser<Expr> {
    s_tag("{")
        .ig_then(repeat(map_item().then_ig(maybe(s_tag(","))), 0))
        .then_ig(s_tag("}"))
        .map(|e| Expr::Map(e))
}

fn code_block() -> impl Parser<Vec<Action>> {
    s_tag("{")
        .ig_then(sep_until(maybe(pp_action), l_break(), s_tag("}")))
        .map(|v| v.into_iter().filter_map(|a| a).collect())
}

fn if_clause() -> impl Parser<Expr> {
    //Todo Consider Elif: Done right, wont even effect anything else
    s_tag("if")
        .ig_then(p_expr)
        .then(code_block())
        .then(maybe(s_tag("else").ig_then(code_block())))
        .map(|((bex, lblk), rop)| {
            if let Some(rblk) = rop {
                Expr::If(Box::new(bex), lblk, rblk)
            } else {
                Expr::If(Box::new(bex), lblk, Vec::new())
            }
        })
}

//must not be impl<Parser<Expr>> to avoid giant objects
fn p_expr_l<'a>(i: &LCChars<'a>) -> ParseRes<'a, Expr> {
    let ps = (tag("true").map(|_| Expr::Bool(true)))
        .or(tag("false").map(|_| Expr::Bool(false)))
        .or(num().map(|v| Expr::Num(v)))
        .or(tag("\"")
            .ig_then(esc('"', '\\').e_map('t', '\t'))
            .map(|s| Expr::Str(s)))
        .or(s_tag(".")
            .ig_then(p_expr)
            .map(|e| Expr::DotStart(Box::new(e))))
        .or(s_tag(":")
            .ig_then(p_expr)
            .map(|e| Expr::Rooted(Box::new(e))))
        .or(s_tag("-").ig_then(p_expr_l).map(|e| Expr::Neg(Box::new(e))))
        .or(s_tag("$").ig_then(p_expr).map(|e| Expr::Deref(Box::new(e))))
        .or(s_tag("(")
            .ig_then(p_expr)
            .then_ig(s_tag(")"))
            .map(|e| Expr::Bracket(Box::new(e))))
        .or(list())
        .or(map())
        .or(if_clause())
        .or(ident().map(|e| Expr::Ident(e)));

    ws(0).ig_then(ps).parse(i)
}

//Cannot be a ->impl Parser() to avoid infinite struct creation
pub fn p_expr<'a>(i: &LCChars<'a>) -> ParseRes<'a, Expr> {
    ws(0)
        .ig_then(p_expr_l)
        .then(maybe(op().then(p_expr)))
        .map(|(l, r)| match r {
            Some((o, re)) => re.add_left(l, o),
            None => l,
        })
        .parse(i)
}
