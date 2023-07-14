UPDATE attribute_type
SET attribute_name = ?1,
    multiple_allowed = ?2,
    last_change_set_id = ?3,
    metadata = ?4
WHERE attribute_type_id = ?5;