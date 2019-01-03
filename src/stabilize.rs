use std::{error::Error, env::current_dir, path::PathBuf};
use crate::file::{Handler, Direction};

#[derive(Clone, Debug)]
pub struct Stabilize {
    feature: String,
    path: PathBuf
}

impl Stabilize {
    pub fn try_new(feature: &str) -> Result<Stabilize, Box<dyn Error>> {
        Ok(Stabilize { feature: feature.to_owned(), path: current_dir()? })
    }

    pub fn start(self) -> Result<(), Box<dyn Error>> {
        self.remove_feature_gate()?;

        Ok(())
    }

    fn remove_feature_gate(&self) -> Result<(), Box<dyn Error>>{
        let search = format!("\\(active, {}", &self.feature);

        Handler::try_new(self.path.join("src/libsyntax/feature_gate.rs"))?.read()?
            .move_n_lines_to("accepted", search.as_str(), 1, Direction::Above)?;

        Ok(())
    }
}

