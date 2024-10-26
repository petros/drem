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
        let permissions = Permissions::from_mode(0o755); // Set permissions to executable by owner, group, and others.
        std::fs::set_permissions(path, permissions).expect("Could not set executable permissions");
    }
}

fn files_exist_in_archive(drgtk: &PathBuf, files: &[&str]) -> bool {
    // Attempt to open the provided archive file.
    let reader = match File::open(drgtk) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Could not open DRGTK: {}", error);
            return false;
        }
    };
    // Create a new ZipArchive from the file reader.
    let mut archive = ZipArchive::new(reader).unwrap();
    let mut result = true;
    // Iterate through the list of files to check if they exist in the archive.
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
    // Check if the provided archive contains the expected DRGTK files.
    files_exist_in_archive(
        drgtk,
        &[
            "dragonruby-macos/dragonruby",
            "dragonruby-macos/console-logo.png",
        ],
    )
}

fn create_gitignore(path: &Path) {
    // Create a .gitignore file in the provided path.
    let mut gitignore =
        File::create(path.join(".gitignore")).expect("Could not create .gitignore file");
    // Write the default content to ignore certain files.
    writeln!(gitignore, ".DS_Store").expect("Could not write to .gitignore file");
}

fn current_directory() -> PathBuf {
    // Get the current working directory.
    std::env::current_dir().unwrap()
}

fn perform_new_command(drgtk: &PathBuf, name: String) -> Result<(), String> {
    // Open the archive file and create a ZipArchive from it.
    let reader = open_archive(drgtk)?;
    let mut archive = create_zip_archive(reader)?;
    // Determine the target directory for extraction.
    let directory = current_directory().join(format!("dragonruby-{}-drgtk", name));
    println!("Extracting to {:?}", directory);
    // Extract the contents of the archive to the target directory.
    extract_archive(&mut archive, &directory)?;
    // Initialize a git repository in the mygame directory.
    initialize_git(&directory)?;
    Ok(())
}

fn open_archive(drgtk: &PathBuf) -> Result<File, String> {
    // Open the provided archive file and return the file handle.
    File::open(drgtk).map_err(|error| format!("Could not open DRGTK: {}", error))
}

fn create_zip_archive(reader: File) -> Result<ZipArchive<File>, String> {
    // Create a ZipArchive from the given file handle.
    ZipArchive::new(reader).map_err(|error| format!("Could not read DRGTK: {}", error))
}

fn extract_archive(archive: &mut ZipArchive<File>, directory: &Path) -> Result<(), String> {
    // Iterate over all files in the ZipArchive.
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|error| format!("Could not read archive file: {}", error))?;
        let outpath = file.mangled_name();

        // Only extract files that start with "dragonruby-macos".
        if (*file.name()).starts_with("dragonruby-macos") {
            // Construct the new path by stripping the prefix and joining with the target directory.
            let new_path =
                Path::new(&directory).join(outpath.strip_prefix("dragonruby-macos").unwrap());
            if (*file.name()).ends_with('/') {
                // If the entry is a directory, create the directory structure.
                create_directory_structure(&new_path)?;
            } else {
                // If the entry is a file, extract it.
                extract_file(&mut file, &new_path)?;
            }
        }
    }
    Ok(())
}

fn create_directory_structure(new_path: &Path) -> Result<(), String> {
    // Create the directory at the specified path.
    std::fs::create_dir_all(&new_path).map_err(|_| "Could not create directory".to_string())?;
    // Create a .gitkeep file for specific directories to ensure they are tracked by git.
    if new_path.ends_with("mygame/data")
        || new_path.ends_with("mygame/fonts")
        || new_path.ends_with("mygame/sounds")
    {
        let gitkeep_path = new_path.join(".gitkeep");
        File::create(gitkeep_path).expect("Could not create .gitkeep file");
    }
    // Create a .gitignore file in the "mygame" directory.
    if new_path.ends_with("mygame") {
        create_gitignore(&new_path);
    }
    Ok(())
}

fn extract_file(file: &mut ZipFile, new_path: &Path) -> Result<(), String> {
    // Ensure that the parent directory exists before creating the file.
    if let Some(p) = new_path.parent() {
        if !p.exists() {
            std::fs::create_dir_all(p)
                .map_err(|_| "Could not create parent directory".to_string())?;
        }
    }
    // Create the output file at the specified path.
    let mut outfile =
        File::create(&new_path).map_err(|_| "Could not create output file".to_string())?;
    // Copy the contents of the archive entry to the output file.
    std::io::copy(file, &mut outfile).map_err(|_| "Could not copy file contents".to_string())?;
    // Set executable permissions if the file is marked as executable.
    set_executable_permissions(file, &new_path);
    Ok(())
}

fn initialize_git(directory: &Path) -> Result<(), String> {
    // Initialize a new git repository in the "mygame" subdirectory.
    git_init(&directory.join("mygame"))
        .map_err(|_| "Could not initialize git repository".to_string())
}

fn git_init(path: &Path) -> Result<(), git2::Error> {
    // Use the git2 library to initialize a git repository at the specified path.
    Repository::init(path)?;
    Ok(())
}

fn build_new_subcommand() -> Command {
    // Define the arguments for the "new" subcommand.
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
    // Build the "new" subcommand with the specified arguments.
    Command::new("new")
        .about("Create a new game")
        .arg(name)
        .arg(drgtk)
}

fn build_command() -> Command {
    // Build the main command for the CLI utility, including the version and subcommands.
    let version = env!("CARGO_PKG_VERSION");
    Command::new("drem")
        .version(version)
        .subcommand(build_new_subcommand())
}

fn main() {
    // Parse command-line arguments and execute the appropriate subcommand.
    let matches = build_command().get_matches();
    if let Some(command) = matches.subcommand_matches("new") {
        if let Some(name) = command.get_one::<String>("name") {
            println!("Creating new game: {}", name);
            if let Some(drgtk) = command.get_one::<PathBuf>("drgtk") {
                println!("Using DRGTK: {:?}", drgtk);
                // Check if the provided DRGTK archive is valid.
                if archive_is_drgtk(drgtk) {
                    // Perform the "new" command to create the game.
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
