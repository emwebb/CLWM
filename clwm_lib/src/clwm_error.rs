use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClwmError {
    #[error("the provided noun could not be found")]
    NounNotFound,
    #[error("the provided noun type could not be found")]
    NounTypeNotFound,
    #[error("the noun type {noun_type:?} already exists")]
    NounTypeAlreadyExists { noun_type: String },
    #[error("the provided noun has no id")]
    NounHasNoId,
    #[error("the provided noun type has no id")]
    NounTypeHasNoId,
    #[error("the data type {data_type:?} already exists")]
    DataTypeAlreadyExists { data_type: String },
    #[error("the provided data type could not be found")]
    DataTypeNotFound,
    #[error("the attribute type {attribute_type:?} already exists")]
    AttributeTypeAlreadyExists { attribute_type: String },
    #[error("the provided attribute type has no id")]
    AttributeTypeHasNoId,
    #[error("the provided attribute type could not be found")]
    AttributeTypeNotFound,
    #[error("the parent noun id or parent attribute id must be set")]
    ParentNounOrParentAttributeIdMustBeSet,
    #[error("the parent noun id and parent attribute id must not both be set")]
    ParentNounAndParentAttributeIdMustNotBeSet,
    #[error("the provided attribute could not be found")]
    AttributeNotFound,
    #[error("the attribute type {attribute_type:?} does not allow multiple attributes")]
    AttributeTypeDoesNotAllowMultipleAttributes { attribute_type: String },
    #[error("the provided attribute has no id")]
    AttributeHasNoId,
    #[error("the provided data type version could not be found")]
    DataTypeVersionNotFound,
    #[error("the provided data does not match the data type definition")]
    DataDoesNotMatchDataTypeDefinition,
    #[error(
        "the provided attribute type id does not match the attribute type id of the attribute"
    )]
    AttributeTypeIdDoesNotMatch,
    #[error("the provided parent noun id does not match the parent noun id of the attribute")]
    ParentNounIdDoesNotMatch,
    #[error(
        "the provided parent attribute id does not match the parent attribute id of the attribute"
    )]
    ParentAttributeIdDoesNotMatch,
}
