mod error;
mod expr;
mod parse;
mod token;
mod dndata;

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
        data.do_action(a?);
    }

    println!("{:?}",data);

    Ok(())
}
