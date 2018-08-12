extern crate filetime;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

use filetime::FileTime;

#[derive(Debug)]
struct SyncFile {
    file_name: OsString,
    path: OsString,
    access: FileTime,
    modified: FileTime,
    size: u64,
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        panic!("Not enough arguments!");
    }

    let source = Path::new(args[1].trim());
    let destination = Path::new(args[2].trim());

    // Make sure that the given arguments are actual directories
    if !source.is_dir() || !destination.is_dir() {
        panic!("Source or destination folder not found");
    }

    let mut all_files: Vec<SyncFile> = Vec::new();

    // Make SyncFiles from the source directory's files and save the data to vector
    for entry in source.read_dir().expect("Reading the directory failed") {
        if let Ok(entry) = entry {
            let file_metadata = fs::metadata(entry.path()).unwrap();

            println!("{:?}", file_metadata.len());

            let temp_file = SyncFile {
                file_name: entry.path().file_name().unwrap().to_os_string(),
                path: entry.path().into_os_string(),
                access: FileTime::from_last_access_time(&file_metadata),
                modified: FileTime::from_last_modification_time(&file_metadata),
                size: file_metadata.len(),
            };

            all_files.push(temp_file);
        }
    }

    // Start copying the files
    for file in &all_files {
        let new_path = Path::new(destination).join(&file.file_name);
        match fs::copy(&file.path, &new_path) {
            Ok(val) => println!("Successfully copied: {}", val),
            Err(err) => println!("Error: {}", err),
        }

        // Set the accessed time and modified time to be the same as on the original file
        match filetime::set_file_times(&new_path, file.access, file.modified) {
            Ok(_) => println!("Successfully modified 'accessed' and 'modified' times"),
            Err(err) => println!("Error: {}", err),
        }
    }

    println!("{:?}", all_files);
}
