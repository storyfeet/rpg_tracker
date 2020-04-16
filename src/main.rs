mod action;
//mod api_funcs;
mod ecs_ish;
mod error;
mod expr;
mod nomp;
//mod prev_iter;
mod proto;
mod scope;
//mod screen;
mod value;

use crate::error::ActionError;
use gobble::{ECode, ParseError};
use scope::Scope;
use std::io::Write;
use std::path::Path;

use clap_conf::prelude::*;

fn main() -> Result<(), failure::Error> {
    let clp = clap_app!(DnDTracker =>
        (about:"Track Dnd Info as it changes")
        (version:crate_version!())
        (author:"Matthew Stoodley")
        (@arg files: -f + takes_value ... "preloadfiles")
        (@arg tracker: -t +takes_value "Working Filename")
        (@arg nogui: -n "No Gui")
    )
    .get_matches();

    let cfg = with_toml_env(&clp, &["/home/games/dnd.toml"]);

    let fname = cfg.grab_local().arg("tracker").done();

    let mut scope = Scope::new();
    if let Some(it) = clp.values_of("files") {
        for fv in it {
            scope.run_file(fv)?;
        }
    }

    if let Some(ref name) = fname {
        scope.run_file(name)?;
    }

    /*    if !clp.is_present("nogui") {
        return screen::run_screen(scope).map_err(|e| e.into());
    }*/

    loop {
        let input = match read_input() {
            Some(s) => s,
            None => break,
        };
        if let Err(e) = scope.handle_input(&input) {
            if let ActionError::ParseErr(ParseError {
                code: ECode::EOF,
                line: _,
                col: _,
            }) = e
            {
            } else {
                println!("{}", e);
            }
        }
    }
    println!("All Done");
    Ok(())
}

pub fn read_input() -> Option<String> {
    print!(">>");
    std::io::stdout().flush().ok();
    let mut input = String::new();
    if let Ok(0) = std::io::stdin().read_line(&mut input) {
        return None;
    }
    if input.chars().next() == Some('<') {
        let mut res = input[1..].to_string();
        loop {
            print!("..   ");
            std::io::stdout().flush().ok();
            let mut input = String::new();
            if let Ok(0) = std::io::stdin().read_line(&mut input) {
                return None;
            }
            match input.chars().next() {
                Some('>') => return Some(res),
                _ => res.push_str(&input),
            }
        }
    }
    return Some(input);
}

pub fn write_action<P: AsRef<Path>>(fname: &Option<P>, s: &str) -> std::io::Result<()> {
    let fname = match fname {
        Some(p) => p,
        None => return Ok(()),
    };

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(fname)?;
    writeln!(f, "{}", s)
}
