use std::{
    path::PathBuf,
    collections::HashMap,
    fs::{self, File, read_dir},
    io::{self, BufReader, BufRead}
};

#[derive(Debug, Clone)]
pub(crate) enum EntryTree {
    Missing,
    File(EntryStatus),
    Directory {
        children: HashMap<PathBuf, EntryTree>,
        status: EntryStatus
    }
}

const IGNORE_LIST: [&'static str; 1] = [".git"];

impl EntryTree {
    
    fn new_dir() -> Self {
        EntryTree::Directory { 
            children: HashMap::new(),
            status: EntryStatus::Unknow
        }
    }

    fn status(&self) -> EntryStatus {
        match self {
            EntryTree::Directory { children: _c, status } => {
                status.clone()
            },
            EntryTree::File(status) => {
                status.clone()
            },
            _ => panic!("Unreachable state"),
        }
    }

    fn set_status(&mut self, new_status: EntryStatus) {
        match self {
            EntryTree::Directory { children: _c, status } => {
                *status = new_status;
            }
            _ => panic!("Unreachable state"),
        }
    }

    fn insert(&mut self, new_path: PathBuf, node: EntryTree) {
        match self {
            EntryTree::Directory { children, status: _s } => {
                children.insert(new_path, node);
            }
            _ => panic!("Unreachable state"),
        }
    }

}

pub enum EntryError {
    Io(io::Error),
    EntryDoesntExist

    // TODO!
}

#[derive(Debug, Clone)]
pub(crate) enum EntryStatus {
    Ok,
    Unknow,
    Different,
    Missing
}

fn copy_dir(src: &PathBuf, dest: &PathBuf) -> io::Result<()> {

    fs::create_dir_all(&dest)?;
    let mut reader = read_dir(&src)?;

    while let Some(entry) = reader.next() {
        let entry = entry?;

        if IGNORE_LIST.iter().any(|&name| name == entry.file_name()) {
            continue;
        }

        let typ = entry.file_type()?;
        let new_dest = dest.clone().join(entry.file_name());

        if typ.is_dir() {
            copy_dir(&entry.path(), &new_dest)?;
        } else {
            fs::copy(entry.path(), new_dest)?;
        }
    }
    Ok(())
}

pub fn update(src: &PathBuf, dest: &PathBuf) -> Result<(), EntryError> {
    if !src.try_exists().map_err(EntryError::Io)? {
        Err(EntryError::EntryDoesntExist)
    } else {
        if src.is_dir() {
            copy_dir(src, dest).map_err(EntryError::Io)?;
        } else {
            fs::copy(src, dest).map_err(EntryError::Io)?;
        }
        Ok(())
    }
}

pub(crate) fn compare_dirs(src: &PathBuf, dest: &PathBuf) -> Result<EntryTree, EntryError> {

    let mut reader = read_dir(&src)
        .map_err(EntryError::Io)?;

    let mut root = EntryTree::new_dir();

    let mut root_status = EntryStatus::Unknow;

    while let Some(entry) = reader.next() {

        let entry = entry.map_err(EntryError::Io)?;

        if IGNORE_LIST.iter().any(|&name| name == entry.file_name()) {
            continue;
        }

        let node;

        let typ = entry.file_type()
            .map_err(EntryError::Io)?;

        let new_dest = &dest.join(entry.file_name());

        let mut status = EntryStatus::Unknow;

        if !new_dest.try_exists().map_err(EntryError::Io)? {
            node = EntryTree::Missing;

        } else if typ.is_dir() {
            node = compare_dirs(&entry.path(), &new_dest)?;
            status = node.status();

        } else {
            status = compare_files(&entry.path(), &new_dest)?;
            node = EntryTree::File(status.clone());
        }

        use EntryStatus::*;
        root_status = match(&root_status, &status) {
            (Unknow, _) | (Ok, _) | (Missing, Different) => status,
            _ => root_status
        };
        root.insert(new_dest.clone(), node);
    }
    root.set_status(root_status.clone());

    Ok(root)
}

pub(crate) fn compare_files(path1: &PathBuf, path2: &PathBuf) -> Result<EntryStatus, EntryError> {

    let f1 = File::open(&path1).map_err(EntryError::Io)?;
    let f2 = File::open(&path2).map_err(EntryError::Io)?;

    let len1 = f1.metadata().map_err(EntryError::Io)?.len();
    let len2 = f2.metadata().map_err(EntryError::Io)?.len();

    let diff = EntryStatus::Different;

    if len1 != len2  {
        return Ok(diff)
    }

    let mut reader1 = BufReader::new(f1);
    let mut reader2 = BufReader::new(f2);

    loop {
        let buf1 = reader1.fill_buf().map_err(EntryError::Io)?;
        let buf2 = reader2.fill_buf().map_err(EntryError::Io)?;

        if buf1.is_empty() && buf2.is_empty() {
            return Ok(EntryStatus::Ok)

        } else if buf1.is_empty() || buf2.is_empty() {
            return Ok(diff)
        }
         
        let min = buf1.len().min(buf2.len());

        if buf1[..min] != buf2[..min] {
            return Ok(diff)
        }

        reader1.consume(min);
        reader2.consume(min);
    }
}

