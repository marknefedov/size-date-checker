use chrono::{DateTime, Datelike, NaiveDate, Utc};
use dashmap::DashMap;
use rayon::prelude::*;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <directory_path>", args[0]);
        std::process::exit(1);
    }
    let folder_path = &args[1];
    let sizes_by_month = DashMap::new();
    process_folder(folder_path, &sizes_by_month)?;
    let mut months: Vec<_> = sizes_by_month
        .iter()
        .map(|e| (*e.key(), *e.value()))
        .collect();
    months.sort_by_key(|&(month, _)| month);
    for (month, size) in months {
        println!("{}: {} bytes", month, size);
    }
    Ok(())
}

fn process_folder<P: AsRef<Path>>(
    folder_path: P,
    sizes_by_month: &DashMap<NaiveDate, u64>,
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
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error reading metadata: {}", e);
                return;
            }
        };
        if metadata.is_file() {
            let modified_time = match metadata.modified() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error getting modified time: {}", e);
                    return;
                }
            };
            let datetime: DateTime<Utc> = modified_time.into();
            let naive_date = NaiveDate::from_ymd_opt(datetime.year(), datetime.month(), 1).unwrap();
            let size = metadata.len();
            sizes_by_month
                .entry(naive_date)
                .and_modify(|total_size| *total_size += size)
                .or_insert(size);
        } else if metadata.is_dir() {
            if let Err(e) = process_folder(entry.path(), sizes_by_month) {
                eprintln!("Error processing folder: {}", e);
            }
        }
    });
    Ok(())
}
