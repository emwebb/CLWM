use anyhow::Ok;
use diffy::create_patch;

use crate::{
    clwm_error::ClwmError,
    clwm_file::ClwmFile,
    data_interface::{DataInterface, DataInterfaceType},
    data_interfaces::data_interface_sqlite::DataInterfaceSQLite,
    model::{DataType, DataTypeDefinition, Noun, NounHistory, NounType, NounTypeHistory},
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
            anyhow::bail!(ClwmError::NounTypeNotFound);
        }

        if !found_noun_types
            .iter()
            .any(|noun_type_record| noun_type_record.noun_type == noun_type)
        {
            anyhow::bail!(ClwmError::NounTypeNotFound);
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

    pub async fn get_all_nouns(&mut self) -> anyhow::Result<Vec<Noun>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_noun_by_all().await?)
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

    pub async fn get_all_noun_types(&mut self) -> anyhow::Result<Vec<NounType>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_noun_type_by_all().await?)
    }

    pub async fn update_noun(&mut self, noun: Noun) -> anyhow::Result<Noun> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        if noun.noun_id.is_none() {
            anyhow::bail!(ClwmError::NounHasNoId);
        }

        let possible_old_noun = transaction.find_noun_by_id(noun.noun_id.unwrap()).await?;
        let old_noun = if possible_old_noun.is_none() {
            anyhow::bail!(ClwmError::NounNotFound);
        } else {
            possible_old_noun.unwrap()
        };

        let new_noun = transaction.update_noun(noun).await?;

        let noun_history = NounHistory {
            noun_id: new_noun.noun_id.unwrap(),
            change_date: None,
            diff_name: create_patch(&old_noun.name, &new_noun.name).to_string(),
            diff_noun_type: create_patch(&old_noun.noun_type, &new_noun.noun_type).to_string(),
            diff_metadata: create_patch(&old_noun.metadata, &new_noun.metadata).to_string(),
        };

        transaction.new_noun_history(noun_history).await?;
        transaction.commit().await?;
        Ok(new_noun)
    }

    pub async fn update_noun_type(&mut self, noun_type: NounType) -> anyhow::Result<NounType> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        if noun_type.noun_type_id.is_none() {
            anyhow::bail!(ClwmError::NounTypeHasNoId);
        }

        let possible_old_noun_type = transaction
            .find_noun_type_by_id(noun_type.noun_type_id.unwrap())
            .await?;

        let old_noun_type = if possible_old_noun_type.is_none() {
            anyhow::bail!(ClwmError::NounTypeNotFound);
        } else {
            possible_old_noun_type.unwrap()
        };

        let new_noun_type = transaction.update_noun_type(noun_type).await?;

        let noun_type_history = NounTypeHistory {
            noun_type_id: new_noun_type.noun_type_id.unwrap(),
            change_date: None,
            diff_noun_type: create_patch(&old_noun_type.noun_type, &new_noun_type.noun_type)
                .to_string(),
            diff_metadata: create_patch(&old_noun_type.metadata, &new_noun_type.metadata)
                .to_string(),
        };
        transaction.new_noun_type_history(noun_type_history).await?;
        transaction.commit().await?;
        Ok(new_noun_type)
    }

    pub async fn get_noun_by_id(&mut self, id: i64) -> anyhow::Result<Option<Noun>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_noun_by_id(id).await?)
    }

    pub async fn get_noun_type_by_id(&mut self, id: i64) -> anyhow::Result<Option<NounType>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_noun_type_by_id(id).await?)
    }

    pub async fn new_data_type(
        &mut self,
        name: String,
        defintion: DataTypeDefinition,
    ) -> anyhow::Result<DataType> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;

        let possible_data_type = transaction
            .find_data_type_latest_by_name(name.clone())
            .await?;

        if let Some(data_type) = possible_data_type {
            if data_type.name == name {
                anyhow::bail!(ClwmError::DataTypeAlreadyExists { data_type: name })
            }
        }

        let created_data_type = transaction
            .new_data_type(DataType {
                name,
                system_defined: false,
                definition: defintion,
                version: Some(1),
                change_date: None,
            })
            .await?;
        transaction.commit().await?;
        Ok(created_data_type)
    }

    pub async fn get_all_data_types(&mut self) -> anyhow::Result<Vec<DataType>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_data_type_all_by_all().await?)
    }

    pub async fn get_latest_data_type_by_name(
        &mut self,
        name: String,
    ) -> anyhow::Result<Option<DataType>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_data_type_latest_by_name(name).await?)
    }

    pub async fn get_all_data_type_by_name(
        &mut self,
        name: String,
    ) -> anyhow::Result<Vec<DataType>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_data_type_all_by_name(name).await?)
    }

    pub async fn update_data_type(&mut self, data_type: DataType) -> anyhow::Result<DataType> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;

        let possible_data_type = transaction
            .find_data_type_latest_by_name(data_type.name.clone())
            .await?;
        if possible_data_type.is_none() {
            anyhow::bail!(ClwmError::DataTypeNotFound)
        } else {
            let old_data_type = possible_data_type.unwrap();
            if old_data_type.name != data_type.name {
                anyhow::bail!(ClwmError::DataTypeNotFound)
            }
            let created_data_type = DataType {
                name: data_type.name.clone(),
                system_defined: false,
                definition: data_type.definition.clone(),
                version: Some(old_data_type.version.unwrap() + 1),
                change_date: None,
            };
            let new_data_type = transaction.new_data_type(created_data_type).await?;
            transaction.commit().await?;
            Ok(new_data_type)
        }
    }
}
