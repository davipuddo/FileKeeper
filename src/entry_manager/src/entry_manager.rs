use std::{
    path::PathBuf,
    collections::HashMap
};

use tokio::{
    fs::{self, File, read_dir},
    io::{self, AsyncBufReadExt, BufReader}
};

#[derive(Debug, Clone)]
enum EntryTree {
    Missing,
    File(EntryStatus),
    Directory {
        children: HashMap<PathBuf, EntryTree>,
        status: EntryStatus
    }
}

impl EntryTree {
    
    fn new_dir() -> Self {
        EntryTree::Directory { 
            children: HashMap::new(),
            status: EntryStatus::Unknow
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

    fn print(&self, n: usize) { 

        match self {
            EntryTree::Directory { children, status: _s } => {
            let padd = "  ".repeat(n);
                for (path, entry) in children {
                    match entry {
                        EntryTree::Directory { children: _c, status: _s } => {
                            println!("{} => {:?}: ", padd, path);
                        }
                        _ => print!("{} => {:?}: ", padd, path)
                    };
                    entry.print(n+1);
                }
            }
            EntryTree::File(status) => {
                println!("{:?}", status); 
            }
            EntryTree::Missing => {
                println!("Missing"); 
            }
        }
    }
}

pub enum EntryError {
    Io(io::Error),

    // TODO!
}

#[derive(Debug, Clone)]
enum EntryStatus {
    Ok,
    Unknow,
    Different
}

pub async fn copy_dir(src: PathBuf, dest: PathBuf) -> io::Result<()> {

    fs::create_dir_all(&dest).await?;
    let mut reader = read_dir(&src).await?;

    while let Some(entry) = reader.next_entry().await? {
        let typ = entry.file_type().await?;
        let new_dest = dest.clone().join(entry.file_name());

        if typ.is_dir() {
            Box::pin(copy_dir(entry.path(), new_dest)).await?;
        } else {
            fs::copy(entry.path(), new_dest).await?;
        }
    }
    Ok(())
}

async fn compare_dirs(src: &PathBuf, dest: &PathBuf) -> Result<EntryTree, EntryError> {

    let mut reader = read_dir(&src)
        .await
        .map_err(EntryError::Io)?;

    let mut root = EntryTree::new_dir();

    while let Some(entry) = reader.next_entry().await.map_err(EntryError::Io)? 
    {
        let node;

        let typ = entry.file_type()
            .await
            .map_err(EntryError::Io)?;

        let new_dest = &dest.join(entry.file_name());

        if !new_dest.try_exists().map_err(EntryError::Io)? {
            node = EntryTree::Missing;

        } else if typ.is_dir() {
            node = Box::pin(compare_dirs(&entry.path(), &new_dest)).await?;

        } else {
            let status = compare_files(&entry.path(), &new_dest).await?;
            node = EntryTree::File(status);
        } 

        root.insert(new_dest.clone(), node);
    }
    Ok(root)
}

async fn compare_files(path1: &PathBuf, path2: &PathBuf) -> Result<EntryStatus, EntryError> {

    let f1 = File::open(&path1).await.map_err(EntryError::Io)?;
    let f2 = File::open(&path2).await.map_err(EntryError::Io)?;

    let len1 = f1.metadata().await.map_err(EntryError::Io)?.len();
    let len2 = f2.metadata().await.map_err(EntryError::Io)?.len();

    let diff = EntryStatus::Different;

    if len1 != len2  {
        return Ok(diff)
    }

    let mut reader1 = BufReader::new(f1);
    let mut reader2 = BufReader::new(f2);

    loop {
        let buf1 = reader1.fill_buf().await.map_err(EntryError::Io)?;
        let buf2 = reader2.fill_buf().await.map_err(EntryError::Io)?;

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

pub async fn compare(src: &PathBuf, dest: &PathBuf) -> Result<(), EntryError> {

    if !src.try_exists().map_err(EntryError::Io)? {
        println!("File [{}] does not exist", src.to_str().unwrap());
        return Ok(());
    }

    if !dest.try_exists().map_err(EntryError::Io)? {
        println!("File [{}] does not exist", dest.to_str().unwrap());
        return Ok(());
    }

    let name = src.file_name().unwrap().to_str().unwrap();

    if (src.is_file() && dest.is_dir())
    || (src.is_dir() && dest.is_file()) {
        print!("Mismatched types");
        Ok(())

    } else if src.is_file() {
        print!("=> {} -- ", name);
        match compare_files(src, dest).await? {
            EntryStatus::Ok => println!("Ok!"),
            EntryStatus::Unknow=> println!("Unknow!"),
            EntryStatus::Different => println!("Entries do not match")
        };
        Ok(())

    } else {        // Is a directory
        println!("==> {}", name);
        compare_dirs(src, dest).await?.print(0);
        Ok(())
    }
}
