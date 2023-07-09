UPDATE attribute
SET attribute_type_id = ?1,
    parent_noun_id = ?2,
    parent_attribute_id = ?3,
    data = ?4,
    data_type_version = ?5,
    metadata = ?6,
    last_change_set_id = ?7
WHERE attribute_id = ?8;