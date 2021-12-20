use is_executable::is_executable;
use std::{env, fs, io, path::PathBuf};

use crate::config::*;

const CONFIG_FILENAMES: [&'static str; 2] = ["x-root.kdl", "x-root.yml"];

pub fn find_root(current_dir: Option<PathBuf>) -> io::Result<(PathBuf, PathBuf)> {
    let current_dir = current_dir.map_or_else(|| env::current_dir(), |path| Ok(path))?;
    let mut ancestors = current_dir.ancestors();
    while let Some(path) = ancestors.next() {
        for config_filename in CONFIG_FILENAMES {
            let config_file = path.join(config_filename);
            if config_file.is_file() {
                return Ok((PathBuf::from(path), config_file));
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Root project configuration not found",
    ))
}

pub fn find_binary(
    hoist_declarations: &Vec<HoistDeclaration>,
    binary_name: &str,
) -> io::Result<Option<PathBuf>> {
    let mut binary = None;
    'outer: for hoist in hoist_declarations {
        match hoist {
            HoistDeclaration::Directory { directory } => {
                for entry in fs::read_dir(directory)? {
                    let entry = entry?;
                    if !entry.file_type()?.is_file() || !is_executable(entry.path()) {
                        continue;
                    }

                    if let Some(filename) = entry.path().file_stem() {
                        if binary_name.eq(filename) {
                            binary = Some(entry.path());
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    Ok(binary)
}
