use crate::file::{Direction, Handler};
use std::{env::current_dir, error::Error, path::PathBuf};
use walkdir::WalkDir;

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
        self.remove_feature_gate()?;
        self.change_conditional_compilation()?;

        Ok(())
    }

    fn remove_feature_gate(&self) -> Result<(), Box<dyn Error>> {
        let search = format!("\\(active, {}", &self.feature);

        let mut handler =
            Handler::try_new(self.path.join("src/libsyntax/feature_gate.rs"))?.read()?;
        let position =
            handler.move_n_lines_to("accepted,", search.as_str(), 1, Direction::Above)?;
        let mut accepted_line = handler
            .nth(position)
            .unwrap()
            .split(',')
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();
        let last_accepted = handler
            .nth(position - 2)
            .unwrap()
            .split(',')
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();
        let last_version = last_accepted
            .iter()
            .find(|token| token.starts_with(" \"1."))
            .unwrap();
        accepted_line[0] = "\t(accepted".to_owned();
        accepted_line[2] = last_version.to_owned();

        let accepted = accepted_line.join(",").to_owned();

        handler.replace_line(position, &accepted)?;

        Ok(())
    }

    fn change_conditional_compilation(&self) -> Result<(), Box<dyn Error>> {
        for entry in WalkDir::new(self.path.join("src")) {



        }
    }
}
