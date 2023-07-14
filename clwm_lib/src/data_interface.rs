use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::model::{
    Attribute, AttributeHistory, AttributeType, AttributeTypeHistory, DataType, Noun, NounHistory,
    NounType, NounTypeHistory,
};

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

    async fn find_data_type_latest_by_name(&self, name: String)
        -> anyhow::Result<Option<DataType>>;

    async fn find_data_type_all_by_name(&self, name: String) -> anyhow::Result<Vec<DataType>>;

    async fn find_data_type_all_by_all(&self) -> anyhow::Result<Vec<DataType>>;

    async fn find_data_type_latest_by_all(&self) -> anyhow::Result<Vec<DataType>>;

    async fn new_attribute_type(
        &self,
        attribute_type: AttributeType,
    ) -> anyhow::Result<AttributeType>;

    async fn new_attribute_type_history(
        &self,
        attribute_type_history: AttributeTypeHistory,
    ) -> anyhow::Result<AttributeTypeHistory>;

    async fn update_attribute_type(
        &self,
        attribute_type: AttributeType,
    ) -> anyhow::Result<AttributeType>;

    async fn find_attribute_type_by_name(&self, name: String)
        -> anyhow::Result<Vec<AttributeType>>;

    async fn find_attribute_type_by_all(&self) -> anyhow::Result<Vec<AttributeType>>;

    async fn find_attribute_type_by_id(&self, id: i64) -> anyhow::Result<Option<AttributeType>>;

    async fn new_attribute(&self, attribute: Attribute) -> anyhow::Result<Attribute>;

    async fn update_attribute(&self, attribute: Attribute) -> anyhow::Result<Attribute>;

    async fn find_attribute_by_all(&self) -> anyhow::Result<Vec<Attribute>>;

    async fn find_attribute_by_id(&self, id: i64) -> anyhow::Result<Option<Attribute>>;

    async fn find_attribute_by_parent_noun_id(
        &self,
        parent_noun_id: i64,
    ) -> anyhow::Result<Vec<Attribute>>;

    async fn find_attribute_by_parent_attribute_id(
        &self,
        parent_attribute_id: i64,
    ) -> anyhow::Result<Vec<Attribute>>;

    async fn find_attribute_by_parent_noun_id_and_attribute_type_id(
        &self,
        parent_noun_id: i64,
        attribute_type_id: i64,
    ) -> anyhow::Result<Vec<Attribute>>;

    async fn find_attribute_by_parent_attribute_id_and_attribute_type_id(
        &self,
        parent_attribute_id: i64,
        attribute_type_id: i64,
    ) -> anyhow::Result<Vec<Attribute>>;

    async fn new_attribute_history(
        &self,
        attribute_history: AttributeHistory,
    ) -> anyhow::Result<AttributeHistory>;
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataInterfaceType {
    Sqlite,
}
