# Rust Folder Utilities

This repository contains two Rust programs for managing files in a folder by their age:

1. `size-date-checker`: Calculates the total size of files in a folder (including nested folders) grouped by month.
2. `date-blacklist-deleter`: Deletes files in a folder (including nested folders) that are older than a specified number of months, while providing an exclusion list to prevent deletion of certain files.

## Prerequisites

To build and run these programs, you need to have Rust and Cargo installed. If you don't have them installed, you can follow the instructions in the [official Rust documentation](https://www.rust-lang.org/tools/install) to install them.

The compiled binaries will be located in the `target/release` folder of each program directory.

## Usage

### 1. size_by_month

Calculate the total size of files in a folder grouped by month:

```
size-date-checker <directory_path>
```

Replace `<directory_path>` with the path to the folder you want to analyze.

### 2. delete_old_files

Delete files in a folder that are older than a specified cutoff date:

```
date-blacklist-deleter <directory_path> <cutoff_date> <do_not_delete_list_file>
```

Replace the following placeholders:

- `<directory_path>`: The path to the folder you want to process.
- `<months>`: The number of months to use as the threshold for file deletion.
- `<do_not_delete_list_file>`: The path to a text file containing absolute file paths that should not be deleted, with one file path per line.
