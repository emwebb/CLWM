SELECT attribute_id,
    attribute_type_id,
    parent_noun_id,
    parent_attribute_id,
    data,
    data_type_version,
    metadata,
    change_date
FROM attribute
    JOIN change_set ON change_set.change_set_id = attribute.last_change_set_id
WHERE attribute.parent_attribute_id = ?1
    AND attribute.attribute_type_id = ?2;