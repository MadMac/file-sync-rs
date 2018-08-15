extern crate filetime;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

use filetime::FileTime;

#[derive(Clone, Debug)]
struct SyncFile {
    file_name: OsString,
    path: OsString,
    access: FileTime,
    modified: FileTime,
    size: u64,
}

#[derive(Debug, PartialEq, Eq)]
enum ActionType {
    CopyFile,
    UpdateFile,
    DeleteFile,
}

#[derive(Debug)]
struct DiffFile {
    file: SyncFile,
    action: ActionType,
    is_directory: bool,
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

    // Stores the information about what actions needs to be done later
    let mut diff_files: Vec<DiffFile> = Vec::new();

    // Vector that stores the paths to the folders that have to be still looked
    let mut folders_todo: Vec<OsString> = Vec::new();
    folders_todo.push(source.as_os_str().to_os_string());

    // Recursively go through all the folders and make the diffs
    loop {
        if folders_todo.len() == 0 {
            break;
        }

        let folder = folders_todo.remove(0);

        let source_path = Path::new(&folder);
        let destination_path_without = source_path.strip_prefix(source).unwrap();
        let destination_path = destination.join(destination_path_without);
 
        let source_files = read_files(source_path, source);

        let mut destination_files: Vec<SyncFile> = read_files(&destination_path, destination);

        let (destination_files_add, mut diff_files_add, mut folders_todo_add) =
            make_diff(source_files, destination_files.to_vec());
        diff_files.append(&mut diff_files_add);
        folders_todo.append(&mut folders_todo_add);
        destination_files = destination_files_add.to_vec();

        for dest_file in &destination_files {
            let temp_diff = DiffFile {
                file: dest_file.clone(),
                action: ActionType::DeleteFile,
                is_directory: Path::new(&dest_file.path).is_dir(),
            };

            diff_files.push(temp_diff);
        }

    }

    // Run the diffs
    for diff in &diff_files {
        println!("Diff: {:?}", &diff);

        match diff.action {
            ActionType::CopyFile | ActionType::UpdateFile => {
                let new_path = Path::new(destination).join(&diff.file.file_name);

                if diff.is_directory {
                    match fs::create_dir(&new_path) {
                        Ok(_) => println!("Successfully made a new dir: {:?}", new_path),
                        Err(err) => println!("Error: {}", err),
                    }
                } else {
                    match fs::copy(&diff.file.path, &new_path) {
                        Ok(_) => println!("Successfully copied: {:?}", diff.file.path),
                        Err(err) => println!("Error: {}", err),
                    }
                }
                // Set the accessed time and modified time to be the same as on the original file
                match filetime::set_file_times(&new_path, diff.file.access, diff.file.modified) {
                    Ok(_) => println!("Successfully modified 'accessed' and 'modified' times"),
                    Err(err) => println!("Error: {}", err),
                }
            }
            ActionType::DeleteFile => {
                let delete_path = Path::new(&diff.file.path);

                // Make sure that the file/dir is still there
                if delete_path.is_file() {
                    println!("DELETE FILE: {:?}", delete_path);

                    match fs::remove_file(delete_path) {
                        Ok(_) => println!("Successfully deleted: {:?}", delete_path),
                        Err(err) => println!("Error: {}", err),
                    }
                }

                if delete_path.is_dir() {
                    println!("DELETE DIRECTORY: {:?}", delete_path);

                    match fs::remove_dir(delete_path) {
                        Ok(_) => println!("Successfully deleted: {:?}", delete_path),
                        Err(err) => println!("Error: {}", err),
                    }
                }
            }
        }
    }

    // println!("{:?}", source_files);
}

// Make SyncFiles from the source directory's files and save the data to vector
fn read_files(directory: &Path, prefix: &Path) -> Vec<SyncFile> {
    let mut result_vec: Vec<SyncFile> = Vec::new();

    if directory.is_dir() {
        for entry in directory.read_dir().expect("Reading the directory failed") {
            if let Ok(entry) = entry {
                let file_metadata = fs::metadata(entry.path()).unwrap();

                let temp_file = SyncFile {
                    file_name: entry
                        .path()
                        .strip_prefix(prefix)
                        .unwrap()
                        .as_os_str()
                        .to_os_string(),
                    path: entry.path().into_os_string(),
                    access: FileTime::from_last_access_time(&file_metadata),
                    modified: FileTime::from_last_modification_time(&file_metadata),
                    size: file_metadata.len(),
                };

                result_vec.push(temp_file);
            }
        }
    }

    result_vec
}

fn make_diff<'a>(
    source_files: Vec<SyncFile>,
    _destination_files: Vec<SyncFile>,
) -> (Vec<SyncFile>, Vec<DiffFile>, Vec<OsString>) {
    let mut diff_files = Vec::new();
    let mut destination_files = _destination_files.to_vec();
    let mut directories_todo = Vec::new();

    // Check the differences between the source folder and the destination
    for file in source_files {
        let mut file_exists_already = false;
        let mut dest_file_index = 0;
        let is_directory = Path::new(&file.path).is_dir();

        println!("Checking file: {:?}", &file.path);

        if is_directory {
            directories_todo.push(file.path.to_os_string());
        }

        // Check if the same file is already in destination
        for (i, dest_file) in destination_files.iter().enumerate() {
            if dest_file.file_name == file.file_name {
                file_exists_already = true;

                // check the modified times and sizes
                if dest_file.size == file.size && dest_file.modified == file.modified {
                    println!("No changes needed: {:?}", &dest_file.path);
                    dest_file_index = i;
                    break;
                } else if file.modified > dest_file.modified {
                    // Update needed
                    let temp_diff = DiffFile {
                        file: file.clone(),
                        action: ActionType::UpdateFile,
                        is_directory: is_directory,
                    };
                    dest_file_index = i;
                    diff_files.push(temp_diff);

                    break;
                }
            }
        }

        if !file_exists_already {
            let temp_diff = DiffFile {
                file: file.clone(),
                action: ActionType::CopyFile,
                is_directory: is_directory,
            };

            diff_files.push(temp_diff);
        } else {
            destination_files.remove(dest_file_index);
        }
    }

    (destination_files, diff_files, directories_todo)
}
