use anyhow::Ok;

use crate::{clwm_file::ClwmFile, data_interface::{DataInterface, DataInterfaceType}, data_interfaces::data_interface_sqlite::DataInterfaceSQLite};

pub struct Clwm {
    pub data_interface : Box<dyn DataInterface>,
    pub clwm_file : ClwmFile
}


impl Clwm {
    pub async fn new(file_name : String) -> anyhow::Result<Clwm> {

        let clwm_file = ClwmFile::load_file(file_name.into())?;
        let mut data_interface : Box<dyn DataInterface> = match &clwm_file.data_interface {
            DataInterfaceType::Sqlite => Box::new(DataInterfaceSQLite::new(clwm_file.url.clone())),
        };

        data_interface.init().await?;

        Ok(Clwm {
            data_interface,
            clwm_file
        })
    }
}