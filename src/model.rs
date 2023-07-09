use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Noun {
    pub noun_id: Option<i64>,
    pub last_changed: Option<DateTime<Utc>>,
    pub name: String,
    pub noun_type: String,
    pub metadata: String,
    pub attributes: Option<Vec<Attribute>>,
}

#[derive(Debug)]
pub struct NounHistory {
    pub noun_id: i64,
    pub change_date: Option<DateTime<Utc>>,
    pub diff_name: String,
    pub diff_noun_type: String,
    pub diff_metadata: String,
}

#[derive(Debug)]
pub struct NounType {
    pub noun_type_id: Option<i64>,
    pub last_changed: Option<DateTime<Utc>>,
    pub noun_type: String,
    pub metadata: String,
}

#[derive(Debug)]
pub struct NounTypeHistory {
    pub noun_type_id: i64,
    pub change_date: Option<DateTime<Utc>>,
    pub diff_noun_type: String,
    pub diff_metadata: String,
}

#[derive(Debug)]
pub struct DataType {
    pub name: String,
    pub system_defined: bool,
    pub definition: DataTypeDefinition,
    pub version: Option<i64>,
    pub change_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DataTypeDefinition {
    Text,
    LongText,
    Boolean,
    Integer,
    Float,
    NounReference,
    Array(Box<DataTypeDefinition>),
    Custom(CustomDataTypeDefinition),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomDataTypeDefinition(pub HashMap<String, DataTypeDefinition>);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DataObject {
    Null,
    Text(String),
    LongText(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    NounReference(i64),
    Array(Vec<DataObject>),
    Custom(CustomDataObject),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CustomDataObject(pub HashMap<String, DataObject>);

#[derive(Debug)]
pub struct AttributeType {
    pub attribute_type_id: Option<i64>,
    pub last_changed: Option<DateTime<Utc>>,
    pub attribute_name: String,
    pub data_type: String,
    pub multiple_allowed: bool,
    pub metadata: String,
}

#[derive(Debug)]
pub struct AttributeTypeHistory {
    pub attribute_type_id: i64,
    pub change_date: Option<DateTime<Utc>>,
    pub diff_attribute_name: String,
    pub diff_multiple_allowed: String,
    pub diff_metadata: String,
}

#[derive(Debug)]
pub struct Attribute {
    pub attribute_id: Option<i64>,
    pub last_changed: Option<DateTime<Utc>>,
    pub attribute_type_id: i64,
    pub parent_noun_id: Option<i64>,
    pub parent_attribute_id: Option<i64>,
    pub data: DataObject,
    pub data_type_version: i64,
    pub metadata: String,
    pub children: Option<Vec<Attribute>>,
}

#[derive(Debug)]
pub struct AttributeHistory {
    pub attribute_id: i64,
    pub change_date: Option<DateTime<Utc>>,
    pub diff_data: String,
    pub diff_data_type_version: String,
    pub diff_metadata: String,
}
