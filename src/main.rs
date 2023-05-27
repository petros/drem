use clap::{Arg, Command, Parser};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author = "Petros Amoiridis", version, about, long_about = None)]
struct Args {}

const DEFAULT_GAME_NAME: &str = "mygame";

fn build_new() -> Command {
    let name = Arg::new("name")
        .short('n')
        .long("name")
        .default_value(DEFAULT_GAME_NAME)
        .help("Name of the new game");
    let drgtk = Arg::new("drgtk")
        .short('g')
        .long("drgtk")
        .required(true)
        .value_parser(clap::value_parser!(PathBuf))
        .help("Path to DRGTK zip to use");
    Command::new("new")
        .about("Create a new game")
        .arg(name)
        .arg(drgtk)
}

fn build_command() -> Command {
    Command::new("drem").subcommand(build_new())
}

fn main() {
    let matches = build_command().get_matches();
    if let Some(command) = matches.subcommand_matches("new") {
        if let Some(name) = command.get_one::<String>("name") {
            println!("Creating new game: {}", name);
        }
        if let Some(drgtk) = command.get_one::<PathBuf>("drgtk") {
            println!("Using DRGTK: {:?}", drgtk);
        }
    }
}
