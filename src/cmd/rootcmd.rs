use clap::{App, Arg, ArgMatches};
use crate::cmd::new_config_cmd;
use crate::interact;
use log::info;

pub fn root_app() -> clap::App<'static> {
    App::new("clisample")
        .version("1.0")
        .author("Shiwen Jia. <jiashiwen@gmail.com>")
        .about("command line sample")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .about("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::new("interact")
            .short('i')
            .long("interact")
            .about("run as interact mod"))
        .arg(Arg::new("v")
            .short('v')
            .multiple_occurrences(true)
            .takes_value(true)
            .about("Sets the level of verbosity"))
        .subcommand(new_config_cmd())
        .subcommand(App::new("test")
            .about("controls testing features")
            .version("1.3")
            .author("Someone E. <someone_else@other.com>")
            .arg(Arg::new("debug")
                .short('d')
                .about("print debug information verbosely")))
}

pub fn run_app() {
    let matches = root_app().get_matches();
    cmd_match(matches);
}

pub fn run_from(args: Vec<String>) {
    match root_app().try_get_matches_from(args.clone()) {
        Ok(matches) => {
            cmd_match(matches);
        }
        Err(err) => {
            err.print().expect("Error writing Error");
            // do_something
        }
    };
}


fn cmd_match(matches: ArgMatches) {
    if matches.is_present("interact") {
        interact::run();
        return;
    }

    // if let Some(c) = matches.value_of("config") {
    //     println!("Value for config: {}", c);
    // }

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

