mod action;
mod api_funcs;
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
        (@arg files: -f + takes_value ... "preloadfiles")
        (@arg tracker: -t +takes_value "Working Filename")
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

    if let Some(ref name)= fname {
        scope.run_file(name)?;
    }

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input == "quit\n" {
            return Ok(());
        }

        for a in parse::ActionReader::new(&input) {
            //println!("--action--{:?}", a);
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
