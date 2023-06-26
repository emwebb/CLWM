use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::{Pool, Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;

use crate::{
    data_interface::{DataInterface, DataInterfaceAccessTransaction},
    model::{Noun, NounHistory, NounType, NounTypeHistory},
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

        let noun_type_record = sqlx::query_file!("sqlite_sqls/noun_type/find/by_id.sql", noun_type_id)
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
}
