use clap::{Arg, Command};
use git2::Repository;
use std::io::Write;
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

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

fn create_gitignore(path: &Path) {
    let mut gitignore =
        File::create(path.join(".gitignore")).expect("Could not create .gitignore file");
    writeln!(gitignore, ".DS_Store").expect("Could not write to .gitignore file");
}

fn current_directory() -> PathBuf {
    std::env::current_dir().unwrap()
}

fn perform_new_command(drgtk: &PathBuf, name: String) -> Result<(), String> {
    let reader = match File::open(drgtk) {
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
        let outpath = file.mangled_name();

        if (*file.name()).starts_with("dragonruby-macos") {
            let new_path =
                Path::new(&directory).join(outpath.strip_prefix("dragonruby-macos").unwrap());

            if (*file.name()).ends_with('/') {
                std::fs::create_dir_all(&new_path).unwrap();
                if new_path.ends_with("mygame/data")
                    || new_path.ends_with("mygame/fonts")
                    || new_path.ends_with("mygame/sounds")
                {
                    let gitkeep_path = new_path.join(".gitkeep");
                    File::create(gitkeep_path).expect("Could not create .gitkeep file");
                }
                if new_path.ends_with("mygame") {
                    // Create the .gitignore file
                    create_gitignore(&new_path);
                }
            } else {
                if let Some(p) = new_path.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = File::create(&new_path).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }
    }
    git_init(&Path::new(&directory).join("mygame")).expect("Could not initialize git repository");
    Ok(())
}

fn git_init(path: &Path) -> Result<(), git2::Error> {
    Repository::init(path)?;
    Ok(())
}

fn build_new_subcommand() -> Command {
    let name = Arg::new("name")
        .short('n')
        .long("name")
        .required(true)
        .help("Name of the new game");
    let drgtk = Arg::new("drgtk")
        .short('d')
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
    Command::new("drem").subcommand(build_new_subcommand())
}

fn main() {
    let matches = build_command().get_matches();
    if let Some(command) = matches.subcommand_matches("new") {
        if let Some(name) = command.get_one::<String>("name") {
            println!("Creating new game: {}", name);
            if let Some(drgtk) = command.get_one::<PathBuf>("drgtk") {
                println!("Using DRGTK: {:?}", drgtk);
                if archive_is_drgtk(drgtk) {
                    match perform_new_command(drgtk, name.clone()) {
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
