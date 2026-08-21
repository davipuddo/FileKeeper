use std::{
    path::PathBuf
};

use tokio::{
    fs::{self, File, read_dir},
    io::{self, AsyncBufReadExt, BufReader}
};

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

pub async fn compare_dirs(src: PathBuf, dest: PathBuf) -> io::Result<bool> {

    let mut reader = read_dir(&src).await?;

    while let Some(entry) = reader.next_entry().await? {
        let typ = entry.file_type().await?;
        let new_dest = dest.clone().join(entry.file_name());
        
        if !new_dest.try_exists()? {
            return Ok(false)
        }

        if typ.is_dir() {
            Box::pin(compare_dirs(entry.path(), new_dest)).await?;
        } else {
            if compare_files(entry.path(), new_dest).await? == false {
                return Ok(false)
            }
        }
    }
    Ok(true)
}

pub async fn compare_files(f1: PathBuf, f2: PathBuf) -> io::Result<bool> {

    let f1 = File::open(&f1).await?;
    let f2 = File::open(&f2).await?;

    if f1.metadata().await?.len() != f2.metadata().await?.len() {
        return Ok(false)
    }

    let mut reader1 = BufReader::new(f1);
    let mut reader2 = BufReader::new(f2);

    loop {
        let buf1 = reader1.fill_buf().await?;
        let buf2 = reader2.fill_buf().await?;

        if buf1.is_empty() && buf2.is_empty() {
            return Ok(true)

        } else if buf1.is_empty() || buf2.is_empty() {
            return Ok(false)
        }
         
        let min = buf1.len().min(buf2.len());

        if buf1[..min] != buf2[..min] {
            return Ok(false)
        }

        reader1.consume(min);
        reader2.consume(min);
    }
}
