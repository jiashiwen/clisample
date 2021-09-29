use rustyline::error::ReadlineError;
use rustyline::Editor;
use shellwords::{split};
use crate::cmd::{root_app, run_from};

pub fn run() {
    println!("readline sample!");
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();

    if rl.load_history("/tmp/history").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match split(line.as_str()).as_mut() {
                    Ok(arg) => {
                        println!("{:?}", arg);
                        arg.insert(0, "clisample".to_string());
                        run_from(arg.to_vec())
                    }
                    Err(err) => {
                        println!("{}", err)
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("/tmp/history").unwrap();
}