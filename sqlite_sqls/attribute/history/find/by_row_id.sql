SELECT attribute_id,
    change_set.change_date,
    diff_data,
    diff_data_type_version,
    diff_metadata
FROM attribute_history
    JOIN change_set ON change_set.change_set_id = attribute_history.change_set_id
WHERE attribute_history.ROWID = ?1