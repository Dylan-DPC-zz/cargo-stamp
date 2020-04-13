use walkdir::WalkDir;
use std::error::Error;
use std::path::PathBuf;
use crate::file::Handler;

pub struct Dir {
    path: PathBuf
}

impl Dir {
    pub fn new(p: PathBuf) -> Dir {
        Dir {
            path: p
        }
    }

    pub fn scan_for(&self, content: &str) -> Result<(), Box<dyn Error>>{
        WalkDir::new(self.path.clone()).into_iter().filter_map(|entry| {
            entry.ok()
        }).filter_map(|entry| {
            Handler::try_new(entry.path()).ok()
        })
            .filter_map(|entry| {
                entry.read().ok()
            })
            .for_each(|mut handler| {
                if let Err(_e) = handler.delete(content) {
                    panic!("cannot delete entry from file");
                }
            });

        Ok(())
    }



}
