INSERT INTO attribute (
        attribute_type_id,
        parent_noun_id,
        parent_attribute_id,
        data,
        data_type_version,
        metadata,
        last_change_set_id
    )
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);