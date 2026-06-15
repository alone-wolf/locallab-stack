mod cert;
mod cli;
mod commands;
mod compose;
mod config;
mod gateway;
mod hosts;
mod manifest;
mod template;
mod workspace;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
