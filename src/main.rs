mod action;
mod error;
mod expr;
mod parse;
mod prev_iter;
mod proto;
mod scope;
mod token;
mod value;

use scope::Scope;
use std::io::Write;
use std::path::Path;

use clap_conf::prelude::*;

fn main() -> Result<(), failure::Error> {
    let clp = clap_app!(DnDTracker =>
        (about:"Track Dnd Info as it changes")
        (version:crate_version!())
        (author:"Matthew Stoodley")
        (@arg file: +required "Working Filename")
    )
    .get_matches();

    let cfg = with_toml_env(&clp, &["/home/games/dnd.toml"]);

    let fname = cfg.grab_local().arg("file").req()?;

    let fs = std::fs::read_to_string(&fname)?;

    let r = parse::ActionReader::new(&fs);

    let mut scope = Scope::new();

    for a in r {
        //        println!(" -- {:?}", a);
        let a = match a {
            Ok(v) => {
                //                println!(" OK {:?}", v);
                v
            }
            Err(e) => {
                println!("Error {}", e);
                continue;
            }
        };
        if let Err(e) = scope.do_action(&a.action) {
            println!("Error {} at {}", e, a.line)
        }
    }

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input == "quit\n" {
            return Ok(());
        }

        for a in parse::ActionReader::new(&input) {
            match a {
                Ok(ac) => match scope.do_action(&ac.action) {
                    Ok(Some(v)) => {
                        if ac.action.is_fileworthy() {
                            write_action(&fname, &input)?;
                        }
                        println!("{}", v.print(0));
                    }
                    Ok(None) => {
                        if ac.action.is_fileworthy() {
                            write_action(&fname, &input)?;
                        }
                    }
                    Err(e) => println!("Error {}", e),
                },
                Err(e) => println!("Error {}", e),
            }
        }
    }
}

pub fn write_action<P: AsRef<Path>>(fname: P, s: &str) -> std::io::Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(fname)?;
    writeln!(f, "{}", s)
}
