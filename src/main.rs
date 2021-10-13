use clap::{App};
use logger::init_log;
// use log::info;


mod cmd;
mod logger;
mod interact;
mod commons;


fn main() {
    init_log();
    cmd::run_app();
}

fn all_subcommands(app: &App, mut level: isize) {
    let mut space = "".to_string();

    for _ in 0..level {
        space.push_str(" ");
    }
    space.push_str("|-");
    let name = String::from(app.get_name());
    space.push_str(name.as_str());
    println!("{}", space);
    if app.has_subcommands() {
        level += 1;
        for val in app.get_subcommands() {
            all_subcommands(val, level)
        }
    }
}

