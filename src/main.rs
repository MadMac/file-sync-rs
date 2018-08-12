use std::env;
use std::time::SystemTime;
use std::path::Path;
use std::fs;
use std::ffi::OsString;

#[derive(Debug)]
struct SyncFile {
    path: OsString,
    modified: SystemTime,
    size: u64
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

    for entry in source.read_dir().expect("Reading the directory failed") {

        if let Ok(entry) = entry {
            let file_metadata = fs::metadata(entry.path()).unwrap();

            println!("{:?}", file_metadata.len());

            let temp_file = SyncFile {
                path: entry.path().into_os_string(),
                modified: file_metadata.modified().unwrap(),
                size: file_metadata.len()
            };

            all_files.push(temp_file);
        }

    }

    println!("{:?}", all_files);

}
