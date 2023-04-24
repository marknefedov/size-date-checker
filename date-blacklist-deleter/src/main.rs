use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use indicatif::{ProgressBar, ProgressStyle};
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
            "Usage: {} <directory_path> <months> <do_not_delete_list_file>",
            args[0]
        );
        std::process::exit(1);
    }
    let folder_path = &args[1];
    let months = args[2].parse::<i64>().expect("Invalid number of months");
    let do_not_delete_list_file = &args[3];
    let do_not_delete_set = load_do_not_delete_set(do_not_delete_list_file)?;
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} Deleting files...")
            .unwrap(),
    );
    delete_old_files(folder_path, months, &pb, &do_not_delete_set)?;
    pb.finish_with_message("Deletion complete.");
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
    months: i64,
    pb: &ProgressBar,
    do_not_delete_set: &HashSet<PathBuf>,
) -> io::Result<()> {
    let entries: Vec<_> = fs::read_dir(folder_path)?.collect();
    let now = Utc::now();
    let duration = Duration::days(30 * months);
    entries.into_par_iter().for_each(|entry| {
        pb.inc_length(1);
        pb.tick();
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
            if now.signed_duration_since(datetime) > duration {
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("Error deleting file: {}", e);
                }
            }
        } else if metadata.is_dir() {
            if let Err(e) = delete_old_files(path, months, pb, do_not_delete_set) {
                eprintln!("Error processing folder: {}", e);
            }
        }
        pb.inc(1);
        pb.tick();
    });
    Ok(())
}
