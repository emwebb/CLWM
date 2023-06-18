use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::data_interface::DataInterfaceType;

#[derive(Serialize, Deserialize, Clone)]
pub struct ClwmFile {
    pub url : String,
    pub data_interface : DataInterfaceType
}

impl ClwmFile {
    pub fn load_file(path : PathBuf) -> anyhow::Result<ClwmFile> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}