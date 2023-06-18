use async_trait::async_trait;
use diffy::create_patch;
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::{data_interface::DataInterface, model::Noun};

pub struct DataInterfaceSQLite {
    url : String,
    connection: Option<Pool<Sqlite>>,
}

impl DataInterfaceSQLite {
    pub fn new(url: String) -> Self {
        DataInterfaceSQLite { url: url, connection: None }
    }
}

#[async_trait]
impl DataInterface for DataInterfaceSQLite {
    async fn init(&mut self) -> anyhow::Result<()> {
        let pool = SqlitePool::connect(&self.url).await?;
        self.connection = Some(pool);
        Ok(())
    }

    async fn new_noun(
        &self,
        name: String,
        noun_type: String,
        metadata: String,
    ) -> anyhow::Result<Noun> {
        let name_diff = create_patch("", &name).to_string();
        let noun_type_diff = create_patch("", &noun_type).to_string();
        let metadata_diff = create_patch("", &metadata).to_string();

        let mut conn = self
            .connection
            .clone()
            .ok_or(anyhow::anyhow!("Not connected to a database!"))?
            .acquire()
            .await?;
        let id = sqlx::query!(
            r#"
                INSERT INTO noun (name, last_updated, noun_type_id, metadata)
                values (?1, unixepoch(), (SELECT noun_type_id FROM noun_type where noun_type = ?2), ?3)
            "#,
            name,
            noun_type,
            metadata
        )
        .execute(&mut conn)
        .await?
        .last_insert_rowid();

        let noun_record = sqlx::query!(
            r#"
                SELECT noun_id, name, noun.last_updated, noun_type, noun.metadata
                FROM noun
                JOIN noun_type on noun_type.noun_type_id = noun.noun_id
                where noun.ROWID = ?1
            "#,
            id
        )
        .fetch_one(&mut conn)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO noun_history (noun_id, change_time, diff_name, diff_noun_type, diff_metadata)
                VALUES (?1, ?2, ?3, ?4, ?5);
            "#,
            noun_record.noun_id,
            noun_record.last_updated,
            name_diff,
            noun_type_diff,
            metadata_diff
        )
        .execute(&mut conn)
        .await?;
        Ok(Noun {
            id: Some(noun_record.noun_id),
            name: noun_record.name,
            noun_type: noun_record.noun_type,
            metadata: noun_record.metadata,
        })
    }

    async fn update_noun(
        &self,
        id: i64,
        name: Option<String>,
        noun_type: Option<String>,
        metadata: Option<String>,
    ) -> anyhow::Result<Noun> {
        let mut conn = self
            .connection
            .clone()
            .ok_or(anyhow::anyhow!("Not connected to a database!"))?
            .acquire()
            .await?;

        let noun_record = sqlx::query!(
            r#"
                SELECT noun_id, name, noun.last_updated, noun_type, noun.metadata
                FROM noun
                JOIN noun_type on noun_type.noun_type_id = noun.noun_id
                where noun_id = ?1;
            "#,
            id
        )
        .fetch_one(&mut conn)
        .await?;

        let new_name = match name {
            Some(new_name) => new_name,
            None => noun_record.name.clone(),
        };

        let new_noun_type = match noun_type {
            Some(new_noun_type) => new_noun_type,
            None => noun_record.noun_type.clone(),
        };

        let new_metadata = match metadata {
            Some(new_metadata) => new_metadata,
            None => noun_record.metadata.clone(),
        };

        let name_diff = create_patch(&noun_record.name, &new_name).to_string();
        let noun_type_diff = create_patch(&noun_record.noun_type, &new_noun_type).to_string();
        let metadata_diff = create_patch(&noun_record.metadata, &new_metadata).to_string();

        let id = sqlx::query!(
            r#"
                UPDATE noun
                SET name = ?1, last_updated = unixepoch(), noun_type_id = (SELECT noun_type_id FROM noun_type where noun_type = ?2), metadata = ?3
                WHERE noun_id = ?4;
            "#,
            new_name,
            new_noun_type,
            new_metadata,
            id
        )
        .execute(&mut conn)
        .await?
        .last_insert_rowid();

        sqlx::query!(
            r#"
                SELECT noun_id, name, noun.last_updated, noun_type, noun.metadata
                FROM noun
                JOIN noun_type on noun_type.noun_type_id = noun.noun_id
                where noun.ROWID = ?1;
            "#,
            id
        )
        .fetch_one(&mut conn)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO noun_history (noun_id, change_time, diff_name, diff_noun_type, diff_metadata)
                VALUES (?1, ?2, ?3, ?4, ?5);
            "#,
            noun_record.noun_id,
            noun_record.last_updated,
            name_diff,
            noun_type_diff,
            metadata_diff
        )
        .execute(&mut conn)
        .await?;
        Ok(Noun {
            id: Some(noun_record.noun_id),
            name: noun_record.name,
            noun_type: noun_record.noun_type,
            metadata: noun_record.metadata,
        })
    }

    async fn find_noun_by_name(
        &self,
        name : String
    ) -> anyhow::Result<Vec<Noun>> {
        let mut conn = self
            .connection
            .clone()
            .ok_or(anyhow::anyhow!("Not connected to a database!"))?
            .acquire()
            .await?;

        let results = sqlx::query!(
            r#"
                SELECT noun_id, name, noun.last_updated, noun_type, noun.metadata
                FROM noun
                JOIN noun_type on noun_type.noun_type_id = noun.noun_id
                WHERE name LIKE "%" || ?1 || "%";
            "#,
            name
        )
        .fetch_all(&mut conn)
        .await?;
        Ok(results.into_iter().map(|noun_record| Noun {
            id: Some(noun_record.noun_id),
            name: noun_record.name,
            noun_type: noun_record.noun_type,
            metadata: noun_record.metadata,
        }).collect())
    }
}
