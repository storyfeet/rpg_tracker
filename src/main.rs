mod error;
mod parse;
mod token;
mod expr;

use clap_conf::prelude::*;
use std::collections::BTreeMap;
use crate::expr::Expr;


#[derive(Debug)]
pub struct DnDItem {
    name: String,
    dtype: String,
    stats: BTreeMap<String, Expr>,
    lists: BTreeMap<String, Vec<String>>,
    items: BTreeMap<String, i32>,
}

impl DnDItem {
    pub fn new(name: String, itype: String) -> Self {
        DnDItem {
            name: name,
            dtype: itype,
            stats: BTreeMap::new(),
            lists: BTreeMap::new(),
            items: BTreeMap::new(),
        }
    }
}

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
     
    for a in r {
        println!(" -- {:?}",a?);
    }


    Ok(())
}
