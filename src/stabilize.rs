
use std::{env::current_dir, error::Error, path::PathBuf};

use crate::dir::Dir;

#[derive(Clone, Debug)]
pub struct Stabilize {
    feature: String,
    path: PathBuf,
}

impl Stabilize {
    pub fn try_new(feature: &str) -> Result<Stabilize, Box<dyn Error>> {
        Ok(Stabilize {
            feature: feature.to_owned(),
            path: current_dir()?,
        })
    }

    pub fn start(self) -> Result<(), Box<dyn Error>> {
        self.remove_feature_flag_from_tests()?;

        Ok(())
    }

    fn remove_feature_flag_from_tests(&self) -> Result<(), Box<dyn Error>> {
        Dir::new(self.path.join("src/test/ui/"))
            .scan_for(format!("#![feature({})]", self.feature).as_str())?;

        Ok(())
    }
}
