#[derive(Debug)]
pub struct Noun {
    pub id : Option<i64>,
    pub name : String,
    pub noun_type : String,
    pub metadata : String
}

#[derive(Debug)]
pub struct NounType {
    pub noun_type : String,
    pub metadata : String
}