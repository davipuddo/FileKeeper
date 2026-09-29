use crate::{ 
    EntryError, EntryStatus, EntryTree,
    compare_dirs, compare_files
};

use std::path::PathBuf;

impl EntryTree {
    fn print_verbose(&self, n: usize) { 
        if let EntryTree::Directory { children, status: _s } = self {
            for (path, entry) in children {
                print!("{}", "  ".repeat(n));
                let name = path.file_name().unwrap().to_str().unwrap();
                match entry {
                    EntryTree::Directory { children: _c, status } => {
                        status_msg(name, status);
                        entry.print_verbose(n+1);
                    },
                    EntryTree::File(status) => {
                        status_msg(name, status);
                    },
                    EntryTree::Missing => {
                        status_msg(name, &EntryStatus::Missing);
                    },
                };
            }
        } else {
            eprintln!("print is meant for directories only");
        }
    }

    fn print(&self) { 
        if let EntryTree::Directory { children, status: _s } = self {
            for (path, entry) in children {
                let name = path.file_name().unwrap().to_str().unwrap();
                match entry {
                    EntryTree::Directory { children: _c, status: _s } => {
                        entry.print();
                    },
                    EntryTree::File(status) => {
                        if ! matches!(status, EntryStatus::Ok) {
                            status_msg(name, status);
                        }
                    },
                    EntryTree::Missing => {
                        status_msg(name, &EntryStatus::Missing);
                    },
                };
            }
        } else {
            eprintln!("print is meant for directories only");
        }
    }
}

fn status_msg(name: &str, status: &EntryStatus) {
    print!("=> {}: ", name);
    match status {
        EntryStatus::Ok => println!("Ok!"),
        EntryStatus::Unknow => println!("Unknow!"),
        EntryStatus::Different => println!("Entries do not match"),
        EntryStatus::Missing => println!("Entries is missing")
    }
}

pub fn compare_checks(src: &PathBuf, dest: &PathBuf, verbose: bool) -> Result<(), EntryError> {
    if !src.try_exists().map_err(EntryError::Io)? {
        eprintln!("File [{}] is missing", src.to_str().unwrap());
        return Ok(());
    }

    if !dest.try_exists().map_err(EntryError::Io)? {
        eprintln!("File [{}] is missing", dest.to_str().unwrap());
        return Ok(());
    }

    if src == dest && verbose {
        println!("Ok!");
    }

    if (src.is_file() && dest.is_dir()) || (src.is_dir() && dest.is_file()) {
        eprintln!("Mismatched types");
        return Ok(())
    }

    Ok(())
}

pub fn compare_verbose(src: &PathBuf, dest: &PathBuf) -> Result<(), EntryError> {

    compare_checks(src, dest, true)?;

    let name = src.file_name().unwrap().to_str().unwrap();

    if src.is_file() {
        let status = compare_files(src, dest)?;
        status_msg(name, &status);

    } else {        // Is a directory
        println!("==> {}", name);
        compare_dirs(src, dest)?.print_verbose(1);
    }
    Ok(())
}

pub fn compare(src: &PathBuf, dest: &PathBuf) -> Result<(), EntryError> {

    compare_checks(src, dest, false)?;

    let name = src.file_name().unwrap().to_str().unwrap();

    if src.is_file() {
        let status = compare_files(src, dest)?;
        if ! matches!(status, EntryStatus::Ok) {
            status_msg(name, &status);
        }

    } else {        // Is a directory
        compare_dirs(src, dest)?.print();
    }
    Ok(())
}
