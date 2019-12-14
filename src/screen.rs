use crate::scope::Scope;
use cursive::Cursive;
use crate::error::ActionError;

pub fn run_screen(s:Scope)->Result<(),ActionError>{
    let mut siv = Cursive::default();

    siv.run();
   
    Ok(())

}

