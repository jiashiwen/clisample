use logger::init_log;

mod cmd;
mod commons;
mod interact;
mod logger;
mod request;

fn main() {
    init_log();
    cmd::run_app();
}
