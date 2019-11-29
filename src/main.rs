mod dndata;
mod error;
mod expr;
mod parse;
mod prev_iter;
mod proto;
mod token;
mod value;

use dndata::DnData;

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

    let mut data = DnData::new();

    for a in r {
        //        println!(" -- {:?}", a);
        let a = match a {
            Ok(v) => {
                println!(" OK {:?}", v);
                v
            }
            Err(e) => {
                println!("Error {}", e);
                continue;
            }
        };
        if let Err(e) = data.do_action(a) {
            println!("Error {}", e);
        }
    }

    println!("{:?}", data);

    Ok(())
}
