use anyhow::Ok;
use async_recursion::async_recursion;
use diffy::create_patch;
use futures::future;

use crate::{
    clwm_error::ClwmError,
    clwm_file::ClwmFile,
    data_interface::{DataInterface, DataInterfaceType, DataInterfaceAccessTransaction},
    data_interfaces::data_interface_sqlite::DataInterfaceSQLite,
    model::{
        Attribute, AttributeHistory, AttributeType, AttributeTypeHistory, DataObject, DataType,
        DataTypeDefinition, Noun, NounHistory, NounType, NounTypeHistory,
    },
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
            attributes: None,
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

    pub async fn new_attribute_type(
        &mut self,
        attribute_name: String,
        multiple_allowed: bool,
        data_type_name: String,
        metadata: String,
    ) -> anyhow::Result<AttributeType> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        let found_attribute_type = transaction
            .find_attribute_type_by_name(attribute_name.clone())
            .await?;
        if found_attribute_type
            .iter()
            .any(|at| at.attribute_name == attribute_name)
        {
            anyhow::bail!(ClwmError::AttributeTypeAlreadyExists {
                attribute_type: attribute_name
            })
        }

        let found_data_type = transaction
            .find_data_type_latest_by_name(data_type_name.clone())
            .await?;
        if found_data_type.is_none() {
            anyhow::bail!(ClwmError::DataTypeNotFound)
        }

        let created_attribute_type = transaction
            .new_attribute_type(AttributeType {
                attribute_type_id: None,
                attribute_name,
                multiple_allowed,
                data_type: data_type_name,
                metadata,
                last_changed: None,
            })
            .await?;

        let attribute_type_history = AttributeTypeHistory {
            attribute_type_id: created_attribute_type.attribute_type_id.unwrap(),
            change_date: None,
            diff_attribute_name: create_patch("", &created_attribute_type.attribute_name)
                .to_string(),
            diff_metadata: create_patch("", &created_attribute_type.metadata).to_string(),
            diff_multiple_allowed: create_patch(
                "",
                &created_attribute_type.multiple_allowed.to_string(),
            )
            .to_string(),
        };
        transaction
            .new_attribute_type_history(attribute_type_history)
            .await?;

        transaction.commit().await?;

        Ok(created_attribute_type)
    }

    pub async fn update_attribute_type(
        &mut self,
        attribute_type: AttributeType,
    ) -> anyhow::Result<AttributeType> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        if attribute_type.attribute_type_id.is_none() {
            anyhow::bail!(ClwmError::AttributeTypeHasNoId)
        }
        let possible_attribute_type = transaction
            .find_attribute_type_by_id(attribute_type.attribute_type_id.unwrap())
            .await?;
        let old_attribute_type = if possible_attribute_type.is_none() {
            anyhow::bail!(ClwmError::AttributeTypeNotFound)
        } else {
            possible_attribute_type.unwrap()
        };

        let new_attribute_type = transaction.new_attribute_type(attribute_type).await?;
        let attribute_type_history = AttributeTypeHistory {
            attribute_type_id: new_attribute_type.attribute_type_id.unwrap(),
            change_date: None,
            diff_attribute_name: create_patch(
                &old_attribute_type.attribute_name,
                &new_attribute_type.attribute_name,
            )
            .to_string(),
            diff_metadata: create_patch(&old_attribute_type.metadata, &new_attribute_type.metadata)
                .to_string(),
            diff_multiple_allowed: create_patch(
                &old_attribute_type.multiple_allowed.to_string(),
                &new_attribute_type.multiple_allowed.to_string(),
            )
            .to_string(),
        };
        transaction
            .new_attribute_type_history(attribute_type_history)
            .await?;
        transaction.commit().await?;
        Ok(new_attribute_type)
    }

    pub async fn get_attribute_type_by_id(
        &mut self,
        attribute_type_id: i64,
    ) -> anyhow::Result<Option<AttributeType>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction
            .find_attribute_type_by_id(attribute_type_id)
            .await?)
    }

    pub async fn get_all_attribute_types(&mut self) -> anyhow::Result<Vec<AttributeType>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_attribute_type_by_all().await?)
    }

    pub async fn new_attribute(
        &mut self,
        attribute_type_id: i64,
        parent_noun_id: Option<i64>,
        parent_attribute_id: Option<i64>,
        data: DataObject,
        data_type_version: i64,
        metadata: String,
    ) -> anyhow::Result<Attribute> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        let found_attribute_type = transaction
            .find_attribute_type_by_id(attribute_type_id)
            .await?;
        if found_attribute_type.is_none() {
            anyhow::bail!(ClwmError::AttributeTypeNotFound)
        }

        if parent_noun_id.is_none() && parent_attribute_id.is_none() {
            anyhow::bail!(ClwmError::ParentNounOrParentAttributeIdMustBeSet)
        }

        if parent_noun_id.is_some() && parent_attribute_id.is_some() {
            anyhow::bail!(ClwmError::ParentNounAndParentAttributeIdMustNotBeSet)
        }

        if parent_noun_id.is_some() {
            let found_noun = transaction.find_noun_by_id(parent_noun_id.unwrap()).await?;
            if found_noun.is_none() {
                anyhow::bail!(ClwmError::NounNotFound)
            }

            if found_attribute_type.as_ref().unwrap().multiple_allowed == false {
                let found_attribute = transaction
                    .find_attribute_by_parent_noun_id_and_attribute_type_id(
                        parent_noun_id.unwrap(),
                        attribute_type_id,
                    )
                    .await?;
                if found_attribute.len() > 0 {
                    anyhow::bail!(ClwmError::AttributeTypeDoesNotAllowMultipleAttributes {
                        attribute_type: found_attribute_type.unwrap().attribute_name
                    })
                }
            }
        }

        if parent_attribute_id.is_some() {
            let found_attribute = transaction
                .find_attribute_by_id(parent_attribute_id.unwrap())
                .await?;
            if found_attribute.is_none() {
                anyhow::bail!(ClwmError::AttributeNotFound)
            }

            if found_attribute_type.as_ref().unwrap().multiple_allowed == false {
                let found_attribute = transaction
                    .find_attribute_by_parent_attribute_id_and_attribute_type_id(
                        parent_attribute_id.unwrap(),
                        attribute_type_id,
                    )
                    .await?;
                if found_attribute.len() > 0 {
                    anyhow::bail!(ClwmError::AttributeTypeDoesNotAllowMultipleAttributes {
                        attribute_type: found_attribute_type.unwrap().attribute_name
                    })
                }
            }
        }

        let found_data_type = transaction
            .find_data_type_all_by_name(found_attribute_type.unwrap().data_type)
            .await?;

        let found_data_type_version = found_data_type
            .iter()
            .find(|&x| x.version == Some(data_type_version));
        if found_data_type_version.is_none() {
            anyhow::bail!(ClwmError::DataTypeVersionNotFound)
        }

        if !is_data_of_data_def(&data, &found_data_type_version.unwrap().definition, true) {
            anyhow::bail!(ClwmError::DataDoesNotMatchDataTypeDefinition)
        }

        let created_attribute = transaction
            .new_attribute(Attribute {
                attribute_id: None,
                attribute_type_id,
                parent_noun_id,
                parent_attribute_id,
                data,
                data_type_version,
                metadata,
                last_changed: None,
                children: None,
            })
            .await?;

        let toml_data = toml::to_string(&created_attribute.data)?;

        let attribute_history = AttributeHistory {
            attribute_id: created_attribute.attribute_id.unwrap(),
            diff_data: create_patch("", &toml_data).to_string(),
            diff_data_type_version: create_patch(
                "",
                created_attribute.data_type_version.to_string().as_str(),
            )
            .to_string(),
            diff_metadata: create_patch("", &created_attribute.metadata).to_string(),
            change_date: None,
        };

        transaction.new_attribute_history(attribute_history).await?;
        transaction.commit().await?;
        Ok(created_attribute)
    }

    pub async fn update_attribute(&mut self, attribute: Attribute) -> anyhow::Result<Attribute> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        if attribute.attribute_id.is_none() {
            anyhow::bail!(ClwmError::AttributeHasNoId)
        }
        let possible_attribute = transaction
            .find_attribute_by_id(attribute.attribute_id.unwrap())
            .await?;
        if possible_attribute.is_none() {
            anyhow::bail!(ClwmError::AttributeNotFound)
        }
        let old_attribute = possible_attribute.unwrap();
        if attribute.attribute_type_id != old_attribute.attribute_type_id {
            anyhow::bail!(ClwmError::AttributeTypeIdDoesNotMatch)
        }
        if attribute.parent_noun_id != old_attribute.parent_noun_id {
            anyhow::bail!(ClwmError::ParentNounIdDoesNotMatch)
        }
        if attribute.parent_attribute_id != old_attribute.parent_attribute_id {
            anyhow::bail!(ClwmError::ParentAttributeIdDoesNotMatch)
        }

        let found_attribute_type = transaction
            .find_attribute_type_by_id(attribute.attribute_type_id)
            .await?;

        let found_data_type = transaction
            .find_data_type_all_by_name(found_attribute_type.unwrap().data_type)
            .await?;

        let found_data_type_version = found_data_type
            .iter()
            .find(|&x| x.version == Some(attribute.data_type_version));
        if found_data_type_version.is_none() {
            anyhow::bail!(ClwmError::DataTypeVersionNotFound)
        }

        if !is_data_of_data_def(&attribute.data, &found_data_type_version.unwrap().definition, true) {
            anyhow::bail!(ClwmError::DataDoesNotMatchDataTypeDefinition)
        }

        let new_attribute = transaction.update_attribute(attribute).await?;

        let toml_data_new = toml::to_string(&new_attribute.data)?;
        let toml_data_old = toml::to_string(&old_attribute.data)?;
        let attribute_history = AttributeHistory{
            attribute_id: new_attribute.attribute_id.unwrap(),
            diff_data: create_patch(&toml_data_old, &toml_data_new).to_string(),
            diff_data_type_version: create_patch(
                old_attribute.data_type_version.to_string().as_str(),
                new_attribute.data_type_version.to_string().as_str(),
            )
            .to_string(),
            diff_metadata: create_patch(&old_attribute.metadata, &new_attribute.metadata).to_string(),
            change_date: None,
        };

        transaction.new_attribute_history(attribute_history).await?;

        transaction.commit().await?;

        Ok(new_attribute)
    }

    pub async fn get_attribute_by_id(
        &mut self,
        attribute_id: i64,
    ) -> anyhow::Result<Option<Attribute>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_attribute_by_id(attribute_id).await?)
    }

    pub async fn get_all_attributes(&mut self) -> anyhow::Result<Vec<Attribute>> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;
        Ok(transaction.find_attribute_by_all().await?)
    }

    pub async fn populate_noun(&mut self, noun: &mut Noun) -> anyhow::Result<()> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;

        self.populate_noun_recursive(noun, &transaction).await?;

        Ok(())
    }

    async fn populate_noun_recursive(&mut self, noun: &mut Noun, transaction : &Box<dyn DataInterfaceAccessTransaction>) -> anyhow::Result<()> {

        let noun_id = match noun.noun_id {
            Some(noun_id) => noun_id,
            None => {
                anyhow::bail!(ClwmError::NounHasNoId)
            }
        };

        let mut found_attributes = transaction
            .find_attribute_by_parent_noun_id(noun_id)
            .await?;

        future::join_all(found_attributes.iter_mut().map(|x| async {
            self.populate_attribute_recursive(x, transaction).await
        }).collect::<Vec<_>>()).await.into_iter().collect::<anyhow::Result<Vec<_>>>()?;

        noun.attributes = Some(found_attributes);
        Ok(())
    }

    pub async fn populate_attribute(&mut self, attribute: &mut Attribute) -> anyhow::Result<()> {
        let transaction = self
            .data_interface
            .create_transaction("CLWM".to_owned())
            .await?;

        self.populate_attribute_recursive(attribute, &transaction).await?;

        Ok(())
    }

    #[async_recursion(?Send)]
    async fn populate_attribute_recursive(&self, attribute: &mut Attribute, transaction : &Box<dyn DataInterfaceAccessTransaction>) -> anyhow::Result<()> {

        let attribute_id = match attribute.attribute_id {
            Some(attribute_id) => attribute_id,
            None => {
                anyhow::bail!(ClwmError::AttributeHasNoId)
            }
        };

        let mut found_attributes = transaction
            .find_attribute_by_parent_attribute_id(attribute_id)
            .await?;

        future::join_all(found_attributes.iter_mut().map(|x| async {
            self.populate_attribute_recursive(x, transaction).await
        }).collect::<Vec<_>>()).await.into_iter().collect::<anyhow::Result<Vec<_>>>()?;

        attribute.children = Some(found_attributes);
        Ok(())
    }
}

fn is_data_of_data_def(
    data: &DataObject,
    data_def: &DataTypeDefinition,
    allow_nulls: bool,
) -> bool {
    if *data == DataObject::Null && allow_nulls == true {
        return true;
    }

    match data_def {
        DataTypeDefinition::Text => {
            if let DataObject::Text(_) = data {
                true
            } else {
                false
            }
        }
        DataTypeDefinition::LongText => {
            if let DataObject::LongText(_) = data {
                true
            } else {
                false
            }
        }
        DataTypeDefinition::Boolean => {
            if let DataObject::Boolean(_) = data {
                true
            } else {
                false
            }
        }
        DataTypeDefinition::Integer => {
            if let DataObject::Integer(_) = data {
                true
            } else {
                false
            }
        }
        DataTypeDefinition::Float => {
            if let DataObject::Float(_) = data {
                true
            } else {
                false
            }
        }
        DataTypeDefinition::NounReference => {
            if let DataObject::NounReference(_) = data {
                true
            } else {
                false
            }
        }
        DataTypeDefinition::Array(array_type) => {
            if let DataObject::Array(array) = data {
                array
                    .iter()
                    .all(|x| is_data_of_data_def(x, array_type, allow_nulls))
            } else {
                false
            }
        }
        DataTypeDefinition::Custom(custom_type) => {
            if let DataObject::Custom(custom) = data {
                custom.0.iter().all(|(key, x)| {
                    custom_type.0.contains_key(key)
                        && is_data_of_data_def(x, &custom_type.0[key], allow_nulls)
                })
            } else {
                false
            }
        }
    }
}
