use std::borrow::Borrow;

use clap::{App, Arg, ArgMatches, Subcommand};
use lazy_static::lazy_static;
use log::info;

use crate::cmd::{new_config_cmd, new_multi_cmd};
use crate::commons::CommandCompleter;
use crate::commons::SubCmd;
use crate::interact;

lazy_static! {
    static ref CLIAPP: clap::App<'static> = App::new("clisample")
        .version("1.0")
        .author("Shiwen Jia. <jiashiwen@gmail.com>")
        .about("command line sample")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .about("Sets a custom config file")
                .takes_value(true)
        )
        .arg(
            Arg::new("interact")
                .short('i')
                .long("interact")
                .about("run as interact mod")
        )
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .takes_value(true)
                .about("Sets the level of verbosity")
        )
        .subcommand(new_config_cmd())
        .subcommand(new_multi_cmd())
        .subcommand(
            App::new("test")
                .about("controls testing features")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .arg(
                    Arg::new("debug")
                        .short('d')
                        .about("print debug information verbosely")
                )
        );
    static ref SUBCMDS: Vec<SubCmd> = subcommands();
}

pub fn run_app() {
    let matches = CLIAPP.clone().get_matches();
    cmd_match(&matches);
}

pub fn run_from(args: Vec<String>) {
    match App::try_get_matches_from(CLIAPP.to_owned(), args.clone()) {
        Ok(matches) => {
            cmd_match(&matches);
        }
        Err(err) => {
            err.print().expect("Error writing Error");
            // do_something
        }
    };
}

// 获取全部子命令，用于构建commandcompleter
pub fn all_subcommand(app: &App, beginlevel: usize, input: &mut Vec<SubCmd>) {
    let nextlevel = beginlevel + 1;
    let mut subcmds = vec![];
    for iterm in app.get_subcommands() {
        subcmds.push(iterm.get_name().to_string());
        if iterm.has_subcommands() {
            all_subcommand(iterm, nextlevel, input);
        }
    }
    let subcommand = SubCmd {
        level: beginlevel,
        command_name: app.get_name().to_string(),
        subcommands: subcmds,
    };
    input.push(subcommand);
}

pub fn get_subcommands() -> Box<[SubCmd]> {
    Box::from(SUBCMDS.clone())
}

pub fn get_CommandCompleter() -> CommandCompleter {
    CommandCompleter::new(SUBCMDS.to_vec())
}

fn subcommands() -> Vec<SubCmd> {
    let mut subcmds = vec![];
    all_subcommand(CLIAPP.clone().borrow(), 0, &mut subcmds);
    subcmds
}

fn cmd_match(matches: &ArgMatches) {
    if matches.is_present("interact") {
        interact::run();
        return;
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match matches.occurrences_of("v") {
        0 => println!("Verbose mode is off"),
        1 => println!("Verbose mode is kind of on"),
        2 => println!("Verbose mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    if let Some(ref matches) = matches.subcommand_matches("test") {
        // "$ myapp test" was run
        if matches.is_present("debug") {
            // "$ myapp test -d" was run
            println!("Printing debug info...");
        } else {
            println!("Printing normally...");
        }
    }

    if let Some(config) = matches.subcommand_matches("config") {
        if let Some(show) = config.subcommand_matches("show") {
            match show.subcommand_name() {
                Some("all") => {
                    println!("config show all");
                    info!("log show all")
                }
                Some("info") => {
                    println!("config show info");
                }
                _ => {}
            }
        }
    }
}
