mod token;
mod parse;
mod error;

use clap_conf::prelude::*;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum DnDType{
    Player,
    Item,
    Weapon,
    Attack,
}



#[derive(Debug)]
pub struct DnDItem {
    name: String,
    dtype:DnDType,
    stats:BTreeMap<String,i32>,
    lists:BTreeMap<String,Vec<String>>,
    items:BTreeMap<String,i32>,
}



impl DnDItem {
    pub fn new(name: String,itype:DnDType) -> Self {
        DnDItem {
            name:name,
            dtype:itype,
            stats:BTreeMap::new(),
            lists:BTreeMap::new(),
            items:BTreeMap::new(),
        }
    }
}

fn main()->Result<(),failure::Error> {

    let clp = clap_app!(DnDTracker => 
            (about:"Track Dnd Info as it changes")
            (version:crate_version!())
            (author:"Matthew Stoodley")
            (@arg file: +required "Working Filename")
        ).get_matches();

    let cfg = with_toml_env(&clp,&["/home/games/dnd.toml"]);

    let fname = cfg.grab_local().arg("file").req()?;

    let f = std::fs::read_to_string(fname)?;

    let mut items: Vec<DnDItem> = Vec::new();

    Ok(())
}
