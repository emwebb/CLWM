use async_trait::async_trait;
use clap::ValueEnum;
use serde::{Serialize, Deserialize};

use crate::model::{Noun, NounType};

#[async_trait]
pub trait DataInterface {
    async fn init(&mut self) -> anyhow::Result<()>;

    async fn new_noun(
        &self,
        name: String,
        noun_type: String,
        metadata: String,
    ) -> anyhow::Result<Noun>;

    async fn update_noun(
        &self,
        id: i64,
        name: Option<String>,
        noun_type: Option<String>,
        metadata: Option<String>,
    ) -> anyhow::Result<Noun>;
    
    async fn find_noun_by_name(
        &self,
        name : String
    ) -> anyhow::Result<Vec<Noun>>;
    /* 
    async fn new_noun_type(
        &self,
        noun_type : String,
        metadata : String
    ) -> anyhow::Result<NounType>;

    async fn update_noun_type(
        &self,
        id: i64,
        noun_type : String,
        metadata : String
    ) -> anyhow::Result<NounType>;

    async fn find_noun_type_by_name(
        &self,
        noun_type : String
    ) -> anyhow::Result<Vec<NounType>>;
    */
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DataInterfaceType {
    Sqlite
}
