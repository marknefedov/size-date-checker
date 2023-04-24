use chrono::{DateTime, Datelike, Duration, Utc};
use rayon::prelude::*;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "Usage: {} <directory_path> <cutoff_date> <do_not_delete_list_file>",
            args[0]
        );
        std::process::exit(1);
    }

    let folder_path = &args[1];
    let cutoff_date = DateTime::parse_from_rfc3339(&args[2])
        .expect("Invalid cutoff date")
        .with_timezone(&Utc);
    let do_not_delete_list_file = &args[3];

    let do_not_delete_set = load_do_not_delete_set(do_not_delete_list_file)?;

    delete_old_files(folder_path, cutoff_date, &do_not_delete_set)?;

    Ok(())
}

fn load_do_not_delete_set<P: AsRef<Path>>(file_path: P) -> io::Result<HashSet<PathBuf>> {
    let file = fs::File::open(file_path)?;
    let buf_reader = io::BufReader::new(file);
    let mut do_not_delete_set = HashSet::new();

    for line in buf_reader.lines() {
        let line = line?;
        let path = PathBuf::from(line.trim());
        do_not_delete_set.insert(path);
    }

    Ok(do_not_delete_set)
}

fn delete_old_files<P: AsRef<Path>>(
    folder_path: P,
    cutoff_date: DateTime<Utc>,
    do_not_delete_set: &HashSet<PathBuf>,
) -> io::Result<()> {
    let entries: Vec<_> = fs::read_dir(folder_path)?.collect();
    entries.into_par_iter().for_each(|entry| {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading directory entry: {}", e);
                return;
            }
        };

        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error reading metadata: {}", e);
                return;
            }
        };

        if metadata.is_file() && !do_not_delete_set.contains(&path) {
            let modified_time = match metadata.modified() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error getting modified time: {}", e);
                    return;
                }
            };

            let datetime: DateTime<Utc> = modified_time.into();
            if datetime < cutoff_date {
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("Error deleting file: {}", e);
                }
            }
        } else if metadata.is_dir() {
            if let Err(e) = delete_old_files(path, cutoff_date, do_not_delete_set) {
                eprintln!("Error processing folder: {}", e);
            }
        }
    });

    Ok(())
}
