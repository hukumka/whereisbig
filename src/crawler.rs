use std::fs::{read_dir, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub struct Crawler {
    treshhold_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirTree {
    pub is_dir: bool,
    pub path: PathBuf,
    pub size: u64,
    pub children: Vec<DirTree>,
}

impl Crawler {
    pub fn new(treshhold_bytes: u64) -> Self {
        Self { treshhold_bytes }
    }

    pub fn walk(&self, path: impl AsRef<Path>) -> io::Result<Vec<DirTree>> {
        let result = self.dir_entry(path)?;
        let result = result
            .into_iter()
            .filter(|file| file.size >= self.treshhold_bytes)
            .collect();
        Ok(result)
    }

    fn dir_entry(&self, path: impl AsRef<Path>) -> io::Result<Vec<DirTree>> {
        let result = read_dir(&path)?
            .into_iter()
            .filter_map(|rentry| self.walk_entry(rentry).ok())
            .collect();
        Ok(result)
    }

    fn walk_entry(&self, entry: io::Result<DirEntry>) -> io::Result<DirTree> {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let children = self.dir_entry(&path)?;
            let size = children.iter().map(|c| c.size).sum();
            let children = children
                .into_iter()
                .filter(|file| file.size >= self.treshhold_bytes)
                .collect();
            Ok(DirTree {
                is_dir: true,
                path: path,
                size,
                children,
            })
        } else {
            Ok(DirTree {
                is_dir: false,
                path: path,
                size: entry.metadata()?.len(),
                children: vec![],
            })
        }
    }
}
