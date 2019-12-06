mod action;
//mod dndata;
mod error;
mod expr;
mod parse;
mod prev_iter;
mod proto;
mod scope;
//mod stack;
mod token;
mod value;

//use dndata::DnData;
use scope::Scope;

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

    let fs = std::fs::read_to_string(fname)?;

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
        if let Err(e) = scope.do_action(a.action) {
            println!("Error {} at {}", e, a.line)
        }
    }

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input == "quit\n" {
            break;
        }

        for a in parse::ActionReader::new(&input) {
            match a {
                Ok(ac) => match scope.do_action(ac.action) {
                    Ok(Some(v)) => println!("{}", v.print(0)),
                    Ok(None) => {}
                    Err(e) => println!("Error {}", e),
                },
                Err(e) => println!("Error {}", e),
            }
        }
    }

    println!("{:?}", scope);

    Ok(())
}
