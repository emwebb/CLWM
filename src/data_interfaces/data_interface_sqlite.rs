use std::sync::Arc;

use anyhow::Error;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::{Pool, Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    data_interface::{DataInterface, DataInterfaceAccessTransaction},
    model::{
        Attribute, AttributeHistory, AttributeType, AttributeTypeHistory, DataType, Noun,
        NounHistory, NounType, NounTypeHistory,
    },
};

pub struct DataInterfaceSQLite {
    url: String,
    connection: Option<Pool<Sqlite>>,
}

impl DataInterfaceSQLite {
    pub fn new(url: String) -> Self {
        DataInterfaceSQLite {
            url: url,
            connection: None,
        }
    }
}

#[async_trait]
impl DataInterface for DataInterfaceSQLite {
    async fn init(&mut self) -> anyhow::Result<()> {
        let pool = SqlitePool::connect(&self.url).await?;
        self.connection = Some(pool);
        Ok(())
    }

    async fn create_transaction(
        &self,
        change_source: String,
    ) -> anyhow::Result<Box<dyn DataInterfaceAccessTransaction>> {
        let mut transaction = self
            .connection
            .clone()
            .ok_or(anyhow::anyhow!("Not connected to a database!"))?
            .begin()
            .await?;
        let id = sqlx::query! {
            r#"
                INSERT INTO change_set (change_date, change_source) VALUES (unixepoch(), ?1)
            "#,
            change_source
        }
        .execute(&mut transaction)
        .await?
        .last_insert_rowid();

        let record = sqlx::query! {
            r#"
                SELECT change_set_id, change_date, change_source FROM change_set
                WHERE ROWID = ?1
            "#,
            id
        }
        .fetch_one(&mut transaction)
        .await?;

        Ok(Box::new(Arc::new(Mutex::new(
            DataInterfaceTransactionSQLite {
                transaction: Some(transaction),
                change_set_id: record.change_set_id,
            },
        ))))
    }
}

struct DataInterfaceTransactionSQLite<'a> {
    transaction: Option<Transaction<'a, Sqlite>>,
    change_set_id: i64,
}

macro_rules! data_transaction {
    ($dit:ident) => {
        $dit.transaction
            .as_mut()
            .ok_or(anyhow::anyhow!("Already taken"))?
    };
}

#[async_trait]
impl DataInterfaceAccessTransaction for Arc<Mutex<DataInterfaceTransactionSQLite<'_>>> {
    async fn commit(&self) -> anyhow::Result<()> {
        self.lock()
            .await
            .transaction
            .take()
            .ok_or(anyhow::anyhow!("Already taken"))?
            .commit()
            .await?;
        Ok(())
    }

    async fn rollback(&self) -> anyhow::Result<()> {
        self.lock()
            .await
            .transaction
            .take()
            .ok_or(anyhow::anyhow!("Already taken"))?
            .rollback()
            .await?;
        Ok(())
    }

    async fn new_noun(&self, noun: Noun) -> anyhow::Result<Noun> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        let id = sqlx::query_file!(
            "sqlite_sqls/noun/new.sql",
            noun.name,
            change_set_id,
            noun.noun_type,
            noun.metadata
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();
        let noun_record = sqlx::query_file!("sqlite_sqls/noun/find/by_row_id.sql", id)
            .fetch_one(data_transaction!(data_interface_transaction))
            .await?;

        Ok(Noun {
            noun_id: Some(noun_record.noun_id),
            last_changed: Some(Utc.timestamp_opt(noun_record.change_date, 0).unwrap()),
            name: noun_record.name,
            noun_type: noun_record.noun_type,
            metadata: noun_record.metadata,
            attributes: None,
        })
    }

    async fn new_noun_history(&self, noun_history: NounHistory) -> anyhow::Result<NounHistory> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        let id = sqlx::query_file!(
            "sqlite_sqls/noun/history/new.sql",
            noun_history.noun_id,
            change_set_id,
            noun_history.diff_name,
            noun_history.diff_noun_type,
            noun_history.diff_metadata
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let noun_history_record =
            sqlx::query_file!("sqlite_sqls/noun/history/find/by_row_id.sql", id)
                .fetch_one(data_transaction!(data_interface_transaction))
                .await?;
        Ok(NounHistory {
            noun_id: noun_history_record.noun_id,
            change_date: Some(
                Utc.timestamp_opt(noun_history_record.change_date, 0)
                    .unwrap(),
            ),
            diff_name: noun_history_record.diff_name,
            diff_noun_type: noun_history_record.diff_noun_type,
            diff_metadata: noun_history_record.diff_metadata,
        })
    }

    async fn update_noun(&self, noun: Noun) -> anyhow::Result<Noun> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        let noun_id = noun.noun_id.ok_or(anyhow::anyhow!("No ID"))?;
        sqlx::query_file!(
            "sqlite_sqls/noun/update.sql",
            noun.name,
            change_set_id,
            noun.noun_type,
            noun.metadata,
            noun_id
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?;

        let noun_record = sqlx::query_file!("sqlite_sqls/noun/find/by_id.sql", noun_id)
            .fetch_one(data_transaction!(data_interface_transaction))
            .await?;

        Ok(Noun {
            noun_id: Some(noun_record.noun_id),
            last_changed: Some(Utc.timestamp_opt(noun_record.change_date, 0).unwrap()),
            name: noun_record.name,
            noun_type: noun_record.noun_type,
            metadata: noun_record.metadata,
            attributes: None,
        })
    }

    async fn find_noun_by_name(&self, name: String) -> anyhow::Result<Vec<Noun>> {
        let mut data_interface_transaction = self.lock().await;

        let noun_records = sqlx::query_file!("sqlite_sqls/noun/find/by_name.sql", name)
            .fetch_all(data_transaction!(data_interface_transaction))
            .await?;

        Ok(noun_records
            .iter()
            .map(|noun_record| Noun {
                noun_id: Some(noun_record.noun_id),
                last_changed: Some(Utc.timestamp_opt(noun_record.change_date, 0).unwrap()),
                name: noun_record.name.to_string(),
                noun_type: noun_record.noun_type.to_string(),
                metadata: noun_record.metadata.to_string(),
                attributes: None,
            })
            .collect())
    }

    async fn find_noun_by_all(&self) -> anyhow::Result<Vec<Noun>> {
        let mut data_interface_transaction = self.lock().await;

        let noun_records = sqlx::query_file!("sqlite_sqls/noun/find/by_all.sql")
            .fetch_all(data_transaction!(data_interface_transaction))
            .await?;

        Ok(noun_records
            .iter()
            .map(|noun_record| Noun {
                noun_id: Some(noun_record.noun_id),
                last_changed: Some(Utc.timestamp_opt(noun_record.change_date, 0).unwrap()),
                name: noun_record.name.to_string(),
                noun_type: noun_record.noun_type.to_string(),
                metadata: noun_record.metadata.to_string(),
                attributes: None,
            })
            .collect())
    }

    async fn find_noun_by_id(&self, id: i64) -> anyhow::Result<Option<Noun>> {
        let mut data_interface_transaction = self.lock().await;

        let possible_noun_record = sqlx::query_file!("sqlite_sqls/noun/find/by_id.sql", id)
            .fetch_optional(data_transaction!(data_interface_transaction))
            .await?;
        match possible_noun_record {
            Some(noun_record) => Ok(Some(Noun {
                noun_id: Some(noun_record.noun_id),
                last_changed: Some(Utc.timestamp_opt(noun_record.change_date, 0).unwrap()),
                name: noun_record.name.to_string(),
                noun_type: noun_record.noun_type.to_string(),
                metadata: noun_record.metadata.to_string(),
                attributes: None,
            })),
            None => Ok(None),
        }
    }

    async fn new_noun_type(&self, noun_type: NounType) -> anyhow::Result<NounType> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        let id = sqlx::query_file!(
            "sqlite_sqls/noun_type/new.sql",
            noun_type.noun_type,
            change_set_id,
            noun_type.metadata
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let noun_type_record = sqlx::query_file!("sqlite_sqls/noun_type/find/by_row_id.sql", id)
            .fetch_one(data_transaction!(data_interface_transaction))
            .await?;

        Ok(NounType {
            noun_type_id: Some(noun_type_record.noun_type_id),
            last_changed: Some(Utc.timestamp_opt(noun_type_record.change_date, 0).unwrap()),
            noun_type: noun_type_record.noun_type,
            metadata: noun_type_record.metadata,
        })
    }

    async fn new_noun_type_history(
        &self,
        noun_type_history: NounTypeHistory,
    ) -> anyhow::Result<NounTypeHistory> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        let id = sqlx::query_file!(
            "sqlite_sqls/noun_type/history/new.sql",
            noun_type_history.noun_type_id,
            change_set_id,
            noun_type_history.diff_noun_type,
            noun_type_history.diff_metadata
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let noun_type_history_record =
            sqlx::query_file!("sqlite_sqls/noun_type/history/find/by_row_id.sql", id)
                .fetch_one(data_transaction!(data_interface_transaction))
                .await?;
        Ok(NounTypeHistory {
            noun_type_id: noun_type_history_record.noun_type_id,
            change_date: Some(
                Utc.timestamp_opt(noun_type_history_record.change_date, 0)
                    .unwrap(),
            ),
            diff_noun_type: noun_type_history_record.diff_noun_type,
            diff_metadata: noun_type_history_record.diff_metadata,
        })
    }

    async fn update_noun_type(&self, noun_type: NounType) -> anyhow::Result<NounType> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        let noun_type_id = noun_type.noun_type_id.ok_or(anyhow::anyhow!("No ID"))?;
        sqlx::query_file!(
            "sqlite_sqls/noun_type/update.sql",
            noun_type.noun_type,
            change_set_id,
            noun_type.metadata,
            noun_type_id
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let noun_type_record =
            sqlx::query_file!("sqlite_sqls/noun_type/find/by_id.sql", noun_type_id)
                .fetch_one(data_transaction!(data_interface_transaction))
                .await?;

        Ok(NounType {
            noun_type_id: Some(noun_type_record.noun_type_id),
            last_changed: Some(Utc.timestamp_opt(noun_type_record.change_date, 0).unwrap()),
            noun_type: noun_type_record.noun_type,
            metadata: noun_type_record.metadata,
        })
    }

    async fn find_noun_type_by_noun_type(
        &self,
        noun_type: String,
    ) -> anyhow::Result<Vec<NounType>> {
        let mut data_interface_transaction = self.lock().await;
        let noun_type_records =
            sqlx::query_file!("sqlite_sqls/noun_type/find/by_noun_type.sql", noun_type)
                .fetch_all(data_transaction!(data_interface_transaction))
                .await?;

        Ok(noun_type_records
            .iter()
            .map(|noun_type_record| NounType {
                noun_type_id: Some(noun_type_record.noun_type_id),
                last_changed: Some(Utc.timestamp_opt(noun_type_record.change_date, 0).unwrap()),
                noun_type: noun_type_record.noun_type.to_string(),
                metadata: noun_type_record.metadata.to_string(),
            })
            .collect())
    }

    async fn find_noun_type_by_all(&self) -> anyhow::Result<Vec<NounType>> {
        let mut data_interface_transaction = self.lock().await;
        let noun_type_records = sqlx::query_file!("sqlite_sqls/noun_type/find/by_all.sql")
            .fetch_all(data_transaction!(data_interface_transaction))
            .await?;

        Ok(noun_type_records
            .iter()
            .map(|noun_type_record| NounType {
                noun_type_id: Some(noun_type_record.noun_type_id),
                last_changed: Some(Utc.timestamp_opt(noun_type_record.change_date, 0).unwrap()),
                noun_type: noun_type_record.noun_type.to_string(),
                metadata: noun_type_record.metadata.to_string(),
            })
            .collect())
    }

    async fn find_noun_type_by_id(&self, noun_type_id: i64) -> anyhow::Result<Option<NounType>> {
        let mut data_interface_transaction = self.lock().await;
        let possible_noun_type_record =
            sqlx::query_file!("sqlite_sqls/noun_type/find/by_id.sql", noun_type_id)
                .fetch_optional(data_transaction!(data_interface_transaction))
                .await?;
        match possible_noun_type_record {
            Some(noun_type_record) => Ok(Some(NounType {
                noun_type_id: Some(noun_type_record.noun_type_id),
                last_changed: Some(Utc.timestamp_opt(noun_type_record.change_date, 0).unwrap()),
                noun_type: noun_type_record.noun_type.to_string(),
                metadata: noun_type_record.metadata.to_string(),
            })),
            None => Ok(None),
        }
    }
    async fn new_data_type(&self, data_type: DataType) -> anyhow::Result<DataType> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;

        let encoded_definition = rmp_serde::to_vec(&data_type.definition)?;

        let id = sqlx::query_file!(
            "sqlite_sqls/data_type/new.sql",
            data_type.name,
            data_type.name,
            data_type.system_defined,
            encoded_definition,
            data_type.version,
            change_set_id
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let data_type_record = sqlx::query_file!("sqlite_sqls/data_type/find/by_row_id.sql", id)
            .fetch_one(data_transaction!(data_interface_transaction))
            .await?;

        Ok(DataType {
            name: data_type_record.data_type_name,
            system_defined: data_type_record.system_defined != 0,
            definition: rmp_serde::from_slice(&data_type_record.definition)?,
            version: Some(data_type_record.version),
            change_date: Some(Utc.timestamp_opt(data_type_record.change_date, 0).unwrap()),
        })
    }

    async fn find_data_type_latest_by_name(
        &self,
        name: String,
    ) -> anyhow::Result<Option<DataType>> {
        let mut data_interface_transaction = self.lock().await;
        let possible_data_type_record =
            sqlx::query_file!("sqlite_sqls/data_type/find/latest_by_name.sql", name)
                .fetch_optional(data_transaction!(data_interface_transaction))
                .await?;
        match possible_data_type_record {
            Some(data_type_record) => Ok(Some(DataType {
                name: data_type_record.data_type_name,
                system_defined: data_type_record.system_defined != 0,
                definition: rmp_serde::from_slice(&data_type_record.definition)?,
                version: Some(data_type_record.version),
                change_date: Some(Utc.timestamp_opt(data_type_record.change_date, 0).unwrap()),
            })),
            None => Ok(None),
        }
    }

    async fn find_data_type_all_by_name(&self, name: String) -> anyhow::Result<Vec<DataType>> {
        let mut data_interface_transaction = self.lock().await;
        let data_type_records =
            sqlx::query_file!("sqlite_sqls/data_type/find/all_by_name.sql", name)
                .fetch_all(data_transaction!(data_interface_transaction))
                .await?;
        Ok(data_type_records
            .iter()
            .map(|data_type_record| {
                Ok::<DataType, Error>(DataType {
                    name: data_type_record.data_type_name.clone(),
                    system_defined: data_type_record.system_defined != 0,
                    definition: rmp_serde::from_slice(&data_type_record.definition)?,
                    version: Some(data_type_record.version),
                    change_date: Some(Utc.timestamp_opt(data_type_record.change_date, 0).unwrap()),
                })
            })
            .collect::<Result<Vec<DataType>, _>>()?)
    }

    async fn find_data_type_all_by_all(&self) -> anyhow::Result<Vec<DataType>> {
        let mut data_interface_transaction = self.lock().await;
        let data_type_records = sqlx::query_file!("sqlite_sqls/data_type/find/all_by_all.sql")
            .fetch_all(data_transaction!(data_interface_transaction))
            .await?;
        Ok(data_type_records
            .iter()
            .map(|data_type_record| {
                Ok::<DataType, Error>(DataType {
                    name: data_type_record.data_type_name.clone(),
                    system_defined: data_type_record.system_defined != 0,
                    definition: rmp_serde::from_slice(&data_type_record.definition)?,
                    version: Some(data_type_record.version),
                    change_date: Some(Utc.timestamp_opt(data_type_record.change_date, 0).unwrap()),
                })
            })
            .collect::<Result<Vec<DataType>, _>>()?)
    }

    async fn find_data_type_latest_by_all(&self) -> anyhow::Result<Vec<DataType>> {
        let mut data_interface_transaction = self.lock().await;
        let data_type_records = sqlx::query_file!("sqlite_sqls/data_type/find/latest_by_all.sql")
            .fetch_all(data_transaction!(data_interface_transaction))
            .await?;
        Ok(data_type_records
            .iter()
            .map(|data_type_record| {
                Ok::<DataType, Error>(DataType {
                    name: data_type_record.data_type_name.clone(),
                    system_defined: data_type_record.system_defined != 0,
                    definition: rmp_serde::from_slice(&data_type_record.definition)?,
                    version: Some(data_type_record.version),
                    change_date: Some(Utc.timestamp_opt(data_type_record.change_date, 0).unwrap()),
                })
            })
            .collect::<Result<Vec<DataType>, _>>()?)
    }

    async fn new_attribute_type(
        &self,
        attribute_type: AttributeType,
    ) -> anyhow::Result<AttributeType> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;

        let id = sqlx::query_file!(
            "sqlite_sqls/attribute_type/new.sql",
            attribute_type.attribute_name,
            attribute_type.data_type,
            attribute_type.multiple_allowed,
            attribute_type.metadata,
            change_set_id
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let attribute_type_record =
            sqlx::query_file!("sqlite_sqls/attribute_type/find/by_row_id.sql", id)
                .fetch_one(data_transaction!(data_interface_transaction))
                .await?;

        Ok(AttributeType {
            attribute_type_id: Some(attribute_type_record.attribute_type_id),
            attribute_name: attribute_type_record.attribute_name,
            data_type: attribute_type_record.data_type_name,
            multiple_allowed: attribute_type_record.multiple_allowed != 0,
            metadata: attribute_type_record.metadata,
            last_changed: Some(
                Utc.timestamp_opt(attribute_type_record.change_date, 0)
                    .unwrap(),
            ),
        })
    }

    async fn new_attribute_type_history(
        &self,
        attribute_type_history: AttributeTypeHistory,
    ) -> anyhow::Result<AttributeTypeHistory> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        let id = sqlx::query_file!(
            "sqlite_sqls/attribute_type/history/new.sql",
            attribute_type_history.attribute_type_id,
            change_set_id,
            attribute_type_history.diff_attribute_name,
            attribute_type_history.diff_multiple_allowed,
            attribute_type_history.diff_metadata
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let attribute_type_history_record =
            sqlx::query_file!("sqlite_sqls/attribute_type/history/find/by_row_id.sql", id)
                .fetch_one(data_transaction!(data_interface_transaction))
                .await?;
        Ok(AttributeTypeHistory {
            attribute_type_id: attribute_type_history_record.attribute_type_id,
            change_date: Some(
                Utc.timestamp_opt(attribute_type_history_record.change_date, 0)
                    .unwrap(),
            ),
            diff_attribute_name: attribute_type_history_record.diff_attribute_name,
            diff_multiple_allowed: attribute_type_history_record.diff_multiple_allowed,
            diff_metadata: attribute_type_history_record.diff_metadata,
        })
    }

    async fn update_attribute_type(
        &self,
        attribute_type: AttributeType,
    ) -> anyhow::Result<AttributeType> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        if attribute_type.attribute_type_id.is_none() {
            return Err(anyhow::anyhow!("Attribute type id is required"));
        };

        let attribute_type_id = attribute_type.attribute_type_id.unwrap();

        sqlx::query_file!(
            "sqlite_sqls/attribute_type/update.sql",
            attribute_type.attribute_name,
            attribute_type.multiple_allowed,
            attribute_type.metadata,
            change_set_id,
            attribute_type_id
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?;

        let attribute_type_record = sqlx::query_file!(
            "sqlite_sqls/attribute_type/find/by_id.sql",
            attribute_type_id
        )
        .fetch_one(data_transaction!(data_interface_transaction))
        .await?;

        Ok(AttributeType {
            attribute_type_id: Some(attribute_type_record.attribute_type_id),
            attribute_name: attribute_type_record.attribute_name,
            data_type: attribute_type_record.data_type_name,
            multiple_allowed: attribute_type_record.multiple_allowed != 0,
            metadata: attribute_type_record.metadata,
            last_changed: Some(
                Utc.timestamp_opt(attribute_type_record.change_date, 0)
                    .unwrap(),
            ),
        })
    }

    async fn find_attribute_type_by_name(
        &self,
        attribute_name: String,
    ) -> anyhow::Result<Vec<AttributeType>> {
        let mut data_interface_transaction = self.lock().await;
        let attribute_type_record = sqlx::query_file!(
            "sqlite_sqls/attribute_type/find/by_name.sql",
            attribute_name
        )
        .fetch_optional(data_transaction!(data_interface_transaction))
        .await?;
        Ok(attribute_type_record
            .iter()
            .map(|attribute_type_record| AttributeType {
                attribute_type_id: Some(attribute_type_record.attribute_type_id),
                attribute_name: attribute_type_record.attribute_name.clone(),
                data_type: attribute_type_record.data_type_name.clone(),
                multiple_allowed: attribute_type_record.multiple_allowed != 0,
                metadata: attribute_type_record.metadata.clone(),
                last_changed: Some(
                    Utc.timestamp_opt(attribute_type_record.change_date, 0)
                        .unwrap(),
                ),
            })
            .collect())
    }

    async fn find_attribute_type_by_all(&self) -> anyhow::Result<Vec<AttributeType>> {
        let mut data_interface_transaction = self.lock().await;
        let attribute_type_record = sqlx::query_file!("sqlite_sqls/attribute_type/find/by_all.sql")
            .fetch_all(data_transaction!(data_interface_transaction))
            .await?;
        Ok(attribute_type_record
            .iter()
            .map(|attribute_type_record| AttributeType {
                attribute_type_id: Some(attribute_type_record.attribute_type_id),
                attribute_name: attribute_type_record.attribute_name.clone(),
                data_type: attribute_type_record.data_type_name.clone(),
                multiple_allowed: attribute_type_record.multiple_allowed != 0,
                metadata: attribute_type_record.metadata.clone(),
                last_changed: Some(
                    Utc.timestamp_opt(attribute_type_record.change_date, 0)
                        .unwrap(),
                ),
            })
            .collect())
    }

    async fn find_attribute_type_by_id(
        &self,
        attribute_type_id: i64,
    ) -> anyhow::Result<Option<AttributeType>> {
        let mut data_interface_transaction = self.lock().await;
        let possible_attribute_type_record = sqlx::query_file!(
            "sqlite_sqls/attribute_type/find/by_id.sql",
            attribute_type_id
        )
        .fetch_optional(data_transaction!(data_interface_transaction))
        .await?;
        match possible_attribute_type_record {
            Some(attribute_type_record) => Ok(Some(AttributeType {
                attribute_type_id: Some(attribute_type_record.attribute_type_id),
                attribute_name: attribute_type_record.attribute_name.clone(),
                data_type: attribute_type_record.data_type_name.clone(),
                multiple_allowed: attribute_type_record.multiple_allowed != 0,
                metadata: attribute_type_record.metadata.clone(),
                last_changed: Some(
                    Utc.timestamp_opt(attribute_type_record.change_date, 0)
                        .unwrap(),
                ),
            })),
            None => Ok(None),
        }
    }

    async fn new_attribute(&self, attribute: Attribute) -> anyhow::Result<Attribute> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;

        let encoded_data = rmp_serde::to_vec(&attribute.data)?;

        let id = sqlx::query_file!(
            "sqlite_sqls/attribute/new.sql",
            attribute.attribute_type_id,
            attribute.parent_noun_id,
            attribute.parent_attribute_id,
            encoded_data,
            attribute.data_type_version,
            attribute.metadata,
            change_set_id
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let attribute_record = sqlx::query_file!("sqlite_sqls/attribute/find/by_row_id.sql", id)
            .fetch_one(data_transaction!(data_interface_transaction))
            .await?;

        Ok(Attribute {
            attribute_id: Some(attribute_record.attribute_id),
            attribute_type_id: attribute_record.attribute_type_id,
            parent_noun_id: attribute_record.parent_noun_id,
            parent_attribute_id: attribute_record.parent_attribute_id,
            data: rmp_serde::from_slice(&attribute_record.data)?,
            data_type_version: attribute_record.data_type_version,
            metadata: attribute_record.metadata,
            last_changed: Some(Utc.timestamp_opt(attribute_record.change_date, 0).unwrap()),
            children: None,
        })
    }

    async fn update_attribute(&self, attribute: Attribute) -> anyhow::Result<Attribute> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;

        let encoded_data = rmp_serde::to_vec(&attribute.data)?;

        sqlx::query_file!(
            "sqlite_sqls/attribute/update.sql",
            attribute.attribute_id,
            attribute.attribute_type_id,
            attribute.parent_noun_id,
            attribute.parent_attribute_id,
            encoded_data,
            attribute.data_type_version,
            attribute.metadata,
            change_set_id
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?;

        let attribute_record = sqlx::query_file!(
            "sqlite_sqls/attribute/find/by_id.sql",
            attribute.attribute_id
        )
        .fetch_one(data_transaction!(data_interface_transaction))
        .await?;

        Ok(Attribute {
            attribute_id: Some(attribute_record.attribute_id),
            attribute_type_id: attribute_record.attribute_type_id,
            parent_noun_id: attribute_record.parent_noun_id,
            parent_attribute_id: attribute_record.parent_attribute_id,
            data: rmp_serde::from_slice(&attribute_record.data)?,
            data_type_version: attribute_record.data_type_version,
            metadata: attribute_record.metadata,
            last_changed: Some(Utc.timestamp_opt(attribute_record.change_date, 0).unwrap()),
            children: None,
        })
    }

    async fn find_attribute_by_all(&self) -> anyhow::Result<Vec<Attribute>> {
        let mut data_interface_transaction = self.lock().await;
        let attribute_record = sqlx::query_file!("sqlite_sqls/attribute/find/by_all.sql")
            .fetch_all(data_transaction!(data_interface_transaction))
            .await?;
        Ok(attribute_record
            .iter()
            .map(|attribute_record| Attribute {
                attribute_id: Some(attribute_record.attribute_id),
                attribute_type_id: attribute_record.attribute_type_id,
                parent_noun_id: attribute_record.parent_noun_id,
                parent_attribute_id: attribute_record.parent_attribute_id,
                data: rmp_serde::from_slice(&attribute_record.data).unwrap(),
                data_type_version: attribute_record.data_type_version,
                metadata: attribute_record.metadata.clone(),
                last_changed: Some(Utc.timestamp_opt(attribute_record.change_date, 0).unwrap()),
                children: None,
            })
            .collect())
    }

    async fn find_attribute_by_id(&self, attribute_id: i64) -> anyhow::Result<Option<Attribute>> {
        let mut data_interface_transaction = self.lock().await;
        let possible_attribute_record =
            sqlx::query_file!("sqlite_sqls/attribute/find/by_id.sql", attribute_id)
                .fetch_optional(data_transaction!(data_interface_transaction))
                .await?;
        match possible_attribute_record {
            Some(attribute_record) => Ok(Some(Attribute {
                attribute_id: Some(attribute_record.attribute_id),
                attribute_type_id: attribute_record.attribute_type_id,
                parent_noun_id: attribute_record.parent_noun_id,
                parent_attribute_id: attribute_record.parent_attribute_id,
                data: rmp_serde::from_slice(&attribute_record.data).unwrap(),
                data_type_version: attribute_record.data_type_version,
                metadata: attribute_record.metadata.clone(),
                last_changed: Some(Utc.timestamp_opt(attribute_record.change_date, 0).unwrap()),
                children: None,
            })),
            None => Ok(None),
        }
    }

    async fn find_attribute_by_parent_noun_id(
        &self,
        parent_noun_id: i64,
    ) -> anyhow::Result<Vec<Attribute>> {
        let mut data_interface_transaction = self.lock().await;
        let attribute_record = sqlx::query_file!(
            "sqlite_sqls/attribute/find/by_parent_noun_id.sql",
            parent_noun_id
        )
        .fetch_all(data_transaction!(data_interface_transaction))
        .await?;
        Ok(attribute_record
            .iter()
            .map(|attribute_record| Attribute {
                attribute_id: Some(attribute_record.attribute_id),
                attribute_type_id: attribute_record.attribute_type_id,
                parent_noun_id: attribute_record.parent_noun_id,
                parent_attribute_id: attribute_record.parent_attribute_id,
                data: rmp_serde::from_slice(&attribute_record.data).unwrap(),
                data_type_version: attribute_record.data_type_version,
                metadata: attribute_record.metadata.clone(),
                last_changed: Some(Utc.timestamp_opt(attribute_record.change_date, 0).unwrap()),
                children: None,
            })
            .collect())
    }

    async fn find_attribute_by_parent_attribute_id(
        &self,
        parent_attribute_id: i64,
    ) -> anyhow::Result<Vec<Attribute>> {
        let mut data_interface_transaction = self.lock().await;
        let attribute_record = sqlx::query_file!(
            "sqlite_sqls/attribute/find/by_parent_attribute_id.sql",
            parent_attribute_id
        )
        .fetch_all(data_transaction!(data_interface_transaction))
        .await?;
        Ok(attribute_record
            .iter()
            .map(|attribute_record| Attribute {
                attribute_id: Some(attribute_record.attribute_id),
                attribute_type_id: attribute_record.attribute_type_id,
                parent_noun_id: attribute_record.parent_noun_id,
                parent_attribute_id: attribute_record.parent_attribute_id,
                data: rmp_serde::from_slice(&attribute_record.data).unwrap(),
                data_type_version: attribute_record.data_type_version,
                metadata: attribute_record.metadata.clone(),
                last_changed: Some(Utc.timestamp_opt(attribute_record.change_date, 0).unwrap()),
                children: None,
            })
            .collect())
    }

    async fn find_attribute_by_parent_noun_id_and_attribute_type_id(
        &self,
        parent_noun_id: i64,
        attribute_type_id: i64,
    ) -> anyhow::Result<Vec<Attribute>> {
        let mut data_interface_transaction = self.lock().await;
        let attribute_record = sqlx::query_file!(
            "sqlite_sqls/attribute/find/by_parent_noun_id_and_attribute_type_id.sql",
            parent_noun_id,
            attribute_type_id
        )
        .fetch_all(data_transaction!(data_interface_transaction))
        .await?;
        Ok(attribute_record
            .iter()
            .map(|attribute_record| Attribute {
                attribute_id: Some(attribute_record.attribute_id),
                attribute_type_id: attribute_record.attribute_type_id,
                parent_noun_id: attribute_record.parent_noun_id,
                parent_attribute_id: attribute_record.parent_attribute_id,
                data_type_version: attribute_record.data_type_version,
                data: rmp_serde::from_slice(&attribute_record.data).unwrap(),
                metadata: attribute_record.metadata.clone(),
                last_changed: Some(Utc.timestamp_opt(attribute_record.change_date, 0).unwrap()),
                children: None,
            })
            .collect())
    }

    async fn find_attribute_by_parent_attribute_id_and_attribute_type_id(
        &self,
        parent_attribute_id: i64,
        attribute_type_id: i64,
    ) -> anyhow::Result<Vec<Attribute>> {
        let mut data_interface_transaction = self.lock().await;
        let attribute_record = sqlx::query_file!(
            "sqlite_sqls/attribute/find/by_parent_attribute_id_and_attribute_type_id.sql",
            parent_attribute_id,
            attribute_type_id
        )
        .fetch_all(data_transaction!(data_interface_transaction))
        .await?;
        Ok(attribute_record
            .iter()
            .map(|attribute_record| Attribute {
                attribute_id: Some(attribute_record.attribute_id),
                attribute_type_id: attribute_record.attribute_type_id,
                parent_noun_id: attribute_record.parent_noun_id,
                parent_attribute_id: attribute_record.parent_attribute_id,
                data: rmp_serde::from_slice(&attribute_record.data).unwrap(),
                data_type_version: attribute_record.data_type_version,
                metadata: attribute_record.metadata.clone(),
                last_changed: Some(Utc.timestamp_opt(attribute_record.change_date, 0).unwrap()),
                children: None,
            })
            .collect())
    }

    async fn new_attribute_history(
        &self,
        attribute_history: AttributeHistory,
    ) -> anyhow::Result<AttributeHistory> {
        let mut data_interface_transaction = self.lock().await;
        let change_set_id = data_interface_transaction.change_set_id;
        let id = sqlx::query_file!(
            "sqlite_sqls/attribute/history/new.sql",
            attribute_history.attribute_id,
            change_set_id,
            attribute_history.diff_data,
            attribute_history.diff_data_type_version,
            attribute_history.diff_metadata
        )
        .execute(data_transaction!(data_interface_transaction))
        .await?
        .last_insert_rowid();

        let attribute_history_record =
            sqlx::query_file!("sqlite_sqls/attribute/history/find/by_row_id.sql", id)
                .fetch_one(data_transaction!(data_interface_transaction))
                .await?;

        Ok(AttributeHistory {
            attribute_id: attribute_history_record.attribute_id,
            diff_data: attribute_history_record.diff_data,
            diff_data_type_version: attribute_history_record.diff_data_type_version,
            diff_metadata: attribute_history_record.diff_metadata,
            change_date: Some(
                Utc.timestamp_opt(attribute_history_record.change_date, 0)
                    .unwrap(),
            ),
        })
    }
}
