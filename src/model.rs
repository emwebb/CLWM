use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Noun {
    pub noun_id: Option<i64>,
    pub last_changed: Option<DateTime<Utc>>,
    pub name: String,
    pub noun_type: String,
    pub metadata: String,
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
