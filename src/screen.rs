use crate::error::ActionError;
//use crate::action::Action;
use crate::scope::Scope;
use cursive::direction::Orientation;
use cursive::traits::*;
use cursive::views::*;
use cursive::Cursive;
use std::cell::RefCell;
use std::rc::Rc;

pub fn run_screen(scope: Scope) -> Result<(), ActionError> {
    let mut siv = Cursive::default();
    let scope = Rc::new(RefCell::new(scope));

    let scope2 = scope.clone();

    let ll = LinearLayout::new(Orientation::Vertical)
        .child(TextView::new("<No results yet>").with_id("opt_console"))
        .child(
            EditView::new()
                .on_submit(move |screen, acs| {
                    loop_actions(screen, &mut scope2.borrow_mut(), acs);
                })
                .with_id("edt_console")
                .fixed_width(30),
        );

    siv.add_layer(Dialog::around(ll).title("Dnd Tracker"));

    siv.run();

    Ok(())
}

pub fn print_message(screen: &mut Cursive, m: &str) {
    screen
        .find_id::<TextView>("opt_console")
        .map(|mut tv| tv.set_content(m));
}

pub fn loop_actions(screen: &mut Cursive, scope: &mut Scope, s: &str) {
    screen
        .find_id::<TextView>("opt_console")
        .map(|mut tv| tv.set_content(format!("Waiting")));
    for a in ActionReader::new(s) {
        match a {
            Ok(ac) => match scope.do_action(&ac.action) {
                Ok(Some(v)) => print_message(screen, &v.print(0)),
                Ok(None) => print_message(screen, "_"),
                Err(e) => print_message(screen, &format!("Error:{}", e)),
            },
            Err(e) => print_message(screen, &format!("Error:{}", e)),
        }
    }
}
