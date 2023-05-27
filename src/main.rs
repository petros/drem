use clap::{Arg, Command, Parser};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

#[derive(Parser, Debug)]
#[command(author = "Petros Amoiridis", version, about, long_about = None)]
struct Args {}

const DEFAULT_GAME_NAME: &str = "mygame";

fn files_exist_in_archive(drgtk: &PathBuf, files: &[&str]) -> bool {
    let reader = File::open(drgtk).unwrap();
    let mut archive = ZipArchive::new(reader).unwrap();
    let mut result = true;
    for file_name in files {
        match archive.by_name(file_name) {
            Ok(_) => {}
            Err(_) => {
                result = false;
            }
        }
    }
    result
}

fn archive_is_drgtk(drgtk: &PathBuf) -> bool {
    files_exist_in_archive(
        drgtk,
        &[
            "dragonruby-macos/dragonruby",
            "dragonruby-macos/console-logo.png",
        ],
    )
}

fn current_directory() -> PathBuf {
    std::env::current_dir().unwrap()
}

fn extract(drgtk: &PathBuf, name: String) -> Result<(), String> {
    let reader = match File::open(dbg!(drgtk)) {
        Ok(file) => file,
        Err(error) => return Err(format!("Could not open DRGTK: {}", error)),
    };
    let mut archive = match ZipArchive::new(reader) {
        Ok(archive) => archive,
        Err(error) => return Err(format!("Could not read DRGTK: {}", error)),
    };
    let directory = current_directory().join(format!("dragonruby-{}-drgtk", name));
    println!("Extracting to {:?}", directory);
    let zip_result = match archive.extract(directory) {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("Could not extract DRGTK: {}", error)),
    };
    zip_result
}

fn unzip(drgtk: &PathBuf, name: String) -> Result<(), String> {
    let reader = match File::open(dbg!(drgtk)) {
        Ok(file) => file,
        Err(error) => return Err(format!("Could not open DRGTK: {}", error)),
    };
    let mut archive = match ZipArchive::new(reader) {
        Ok(archive) => archive,
        Err(error) => return Err(format!("Could not read DRGTK: {}", error)),
    };
    let directory = current_directory().join(format!("dragonruby-{}-drgtk", name));
    println!("Extracting to {:?}", directory);
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(error) => return Err(format!("Could not read archive file: {}", error)),
        };
        let outpath = file.sanitized_name();

        if (&*file.name()).starts_with("dragonruby-macos") {
            let new_path = Path::new(format!("dragonruby-{}-drgtk", name).as_str())
                .join(outpath.strip_prefix("dragonruby-macos").unwrap());

            if (&*file.name()).ends_with('/') {
                std::fs::create_dir_all(&new_path).unwrap();
            } else {
                if let Some(p) = new_path.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = std::fs::File::create(&new_path).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }
    }
    Ok(())
}

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
            if let Some(drgtk) = command.get_one::<PathBuf>("drgtk") {
                println!("Using DRGTK: {:?}", drgtk);
                if archive_is_drgtk(drgtk) {
                    match unzip(drgtk, name.clone()) {
                        Ok(_) => {
                            println!("Done!");
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                } else {
                    println!("DRGTK is not a valid macOS archive");
                }
            }
        }
    }
}
