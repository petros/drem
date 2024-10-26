use clap::{Arg, Command};
use git2::Repository;
use std::fs::{File, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use zip::read::ZipFile;
use zip::ZipArchive;

fn set_executable_permissions(zip_file: &ZipFile, path: &Path) {
    // Check if the file is supposed to be executable.
    // You can identify executables based on naming conventions or by checking the file attributes.
    if zip_file.unix_mode().unwrap_or(0) & 0o111 != 0 {
        // If the original file had any executable bit set, set the executable permissions.
        let permissions = Permissions::from_mode(0o755); // Adjust mode as needed
        std::fs::set_permissions(path, permissions).expect("Could not set executable permissions");
    }
}

fn files_exist_in_archive(drgtk: &PathBuf, files: &[&str]) -> bool {
    let reader = match File::open(drgtk) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Could not open DRGTK: {}", error);
            return false;
        }
    };
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
    let reader = open_archive(drgtk)?;
    let mut archive = create_zip_archive(reader)?;
    let directory = current_directory().join(format!("dragonruby-{}-drgtk", name));
    println!("Extracting to {:?}", directory);
    extract_archive(&mut archive, &directory)?;
    initialize_git(&directory)?;
    Ok(())
}

fn open_archive(drgtk: &PathBuf) -> Result<File, String> {
    File::open(drgtk).map_err(|error| format!("Could not open DRGTK: {}", error))
}

fn create_zip_archive(reader: File) -> Result<ZipArchive<File>, String> {
    ZipArchive::new(reader).map_err(|error| format!("Could not read DRGTK: {}", error))
}

fn extract_archive(archive: &mut ZipArchive<File>, directory: &Path) -> Result<(), String> {
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|error| format!("Could not read archive file: {}", error))?;
        let outpath = file.mangled_name();

        if (*file.name()).starts_with("dragonruby-macos") {
            let new_path =
                Path::new(&directory).join(outpath.strip_prefix("dragonruby-macos").unwrap());
            if (*file.name()).ends_with('/') {
                create_directory_structure(&new_path)?;
            } else {
                extract_file(&mut file, &new_path)?;
            }
        }
    }
    Ok(())
}

fn create_directory_structure(new_path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(&new_path).map_err(|_| "Could not create directory".to_string())?;
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
    Ok(())
}

fn extract_file(file: &mut ZipFile, new_path: &Path) -> Result<(), String> {
    if let Some(p) = new_path.parent() {
        if !p.exists() {
            std::fs::create_dir_all(p)
                .map_err(|_| "Could not create parent directory".to_string())?;
        }
    }
    let mut outfile =
        File::create(&new_path).map_err(|_| "Could not create output file".to_string())?;
    std::io::copy(file, &mut outfile).map_err(|_| "Could not copy file contents".to_string())?;
    // Set executable permission if necessary
    set_executable_permissions(file, &new_path);
    Ok(())
}

fn initialize_git(directory: &Path) -> Result<(), String> {
    git_init(&directory.join("mygame"))
        .map_err(|_| "Could not initialize git repository".to_string())
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
    let version = env!("CARGO_PKG_VERSION");
    Command::new("drem")
        .version(version)
        .subcommand(build_new_subcommand())
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
                            std::process::exit(1);
                        }
                    }
                } else {
                    println!("DRGTK is not a valid macOS archive");
                    std::process::exit(1);
                }
            }
        }
    }
}
