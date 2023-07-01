use async_trait::async_trait;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::model::{DataType, Noun, NounHistory, NounType, NounTypeHistory};

#[async_trait]
pub trait DataInterface {
    async fn init(&mut self) -> anyhow::Result<()>;

    async fn create_transaction(
        &self,
        change_source: String,
    ) -> anyhow::Result<Box<dyn DataInterfaceAccessTransaction>>;
}

#[async_trait]
pub trait DataInterfaceAccessTransaction {
    async fn commit(&self) -> anyhow::Result<()>;
    async fn rollback(&self) -> anyhow::Result<()>;

    async fn new_noun(&self, noun: Noun) -> anyhow::Result<Noun>;

    async fn new_noun_history(&self, noun_history: NounHistory) -> anyhow::Result<NounHistory>;

    async fn update_noun(&self, noun: Noun) -> anyhow::Result<Noun>;

    async fn find_noun_by_name(&self, name: String) -> anyhow::Result<Vec<Noun>>;

    async fn find_noun_by_all(&self) -> anyhow::Result<Vec<Noun>>;

    async fn find_noun_by_id(&self, id: i64) -> anyhow::Result<Option<Noun>>;

    async fn new_noun_type(&self, noun_type: NounType) -> anyhow::Result<NounType>;

    async fn new_noun_type_history(
        &self,
        noun_type_history: NounTypeHistory,
    ) -> anyhow::Result<NounTypeHistory>;

    async fn update_noun_type(&self, noun_type: NounType) -> anyhow::Result<NounType>;

    async fn find_noun_type_by_noun_type(&self, noun_type: String)
        -> anyhow::Result<Vec<NounType>>;

    async fn find_noun_type_by_all(&self) -> anyhow::Result<Vec<NounType>>;

    async fn find_noun_type_by_id(&self, noun_type_id: i64) -> anyhow::Result<Option<NounType>>;

    async fn new_data_type(&self, data_type: DataType) -> anyhow::Result<DataType>;

    async fn find_data_type_latest_by_name(&self, name: String) -> anyhow::Result<Option<DataType>>;

    async fn find_data_type_all_by_name(&self, name: String) -> anyhow::Result<Vec<DataType>>;

    async fn find_data_type_all_by_all(&self) -> anyhow::Result<Vec<DataType>>;

    async fn find_data_type_latest_by_all(&self) -> anyhow::Result<Vec<DataType>>;
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DataInterfaceType {
    Sqlite,
}
