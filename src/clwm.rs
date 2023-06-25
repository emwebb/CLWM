use anyhow::Ok;
use diffy::create_patch;

use crate::{
    clwm_error::ClwmError,
    clwm_file::ClwmFile,
    data_interface::{DataInterface, DataInterfaceType},
    data_interfaces::data_interface_sqlite::DataInterfaceSQLite,
    model::{Noun, NounHistory, NounType, NounTypeHistory},
};

pub struct Clwm {
    pub data_interface: Box<dyn DataInterface>,
    pub clwm_file: ClwmFile,
}

impl Clwm {
    pub async fn new(file_name: String) -> anyhow::Result<Clwm> {
        let clwm_file = ClwmFile::load_file(file_name.into())?;
        let mut data_interface: Box<dyn DataInterface> = match &clwm_file.data_interface {
            DataInterfaceType::Sqlite => Box::new(DataInterfaceSQLite::new(clwm_file.url.clone())),
        };

        data_interface.init().await?;

        Ok(Clwm {
            data_interface,
            clwm_file,
        })
    }

    pub async fn create(
        data_interface_type: DataInterfaceType,
        url: String,
        file_name: String,
    ) -> anyhow::Result<()> {
        let file = ClwmFile {
            url,
            data_interface: data_interface_type,
        };
        file.save_file(file_name.into())?;
        Ok(())
    }

    pub async fn new_noun(
        &mut self,
        name: String,
        noun_type: String,
        metadata: String,
    ) -> anyhow::Result<Noun> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;

        let found_noun_types = transaction
            .find_noun_type_by_noun_type(noun_type.clone())
            .await?;
        if found_noun_types.is_empty() {
            anyhow::bail!(ClwmError::NounTypeNotFound {
                noun_type: noun_type.clone()
            });
        }

        if !found_noun_types
            .iter()
            .any(|noun_type_record| noun_type_record.noun_type == noun_type)
        {
            anyhow::bail!(ClwmError::NounTypeNotFound {
                noun_type: noun_type.clone()
            });
        }

        let new_noun = Noun {
            noun_id: None,
            last_changed: None,
            name,
            noun_type,
            metadata,
        };

        let created_noun = transaction.new_noun(new_noun).await?;
        let noun_history = NounHistory {
            noun_id: created_noun.noun_id.unwrap(),
            change_date: None,
            diff_name: create_patch("", &created_noun.name).to_string(),
            diff_noun_type: create_patch("", &created_noun.noun_type).to_string(),
            diff_metadata: create_patch("", &created_noun.metadata).to_string(),
        };

        transaction.new_noun_history(noun_history).await?;

        transaction.commit().await?;
        Ok(created_noun)
    }

    pub async fn new_noun_type(
        &mut self,
        noun_type: String,
        metadata: String,
    ) -> anyhow::Result<NounType> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        let found_noun_types = transaction
            .find_noun_type_by_noun_type(noun_type.clone())
            .await?;
        if found_noun_types
            .iter()
            .any(|noun_type_record| noun_type_record.noun_type == noun_type)
        {
            anyhow::bail!(ClwmError::NounTypeAlreadyExists {
                noun_type: noun_type.clone()
            })
        };
        let new_noun_type = NounType {
            noun_type_id: None,
            last_changed: None,
            noun_type,
            metadata: metadata,
        };
        let created_noun_type = transaction.new_noun_type(new_noun_type).await?;
        let noun_type_history = NounTypeHistory {
            noun_type_id: created_noun_type.noun_type_id.unwrap(),
            change_date: None,
            diff_noun_type: create_patch("", &created_noun_type.noun_type).to_string(),
            diff_metadata: create_patch("", &created_noun_type.metadata).to_string(),
        };
        transaction.new_noun_type_history(noun_type_history).await?;

        transaction.commit().await?;
        Ok(created_noun_type)
    }
}
