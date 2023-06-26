SELECT noun_type_id,
    noun_type,
    change_set.change_date,
    metadata
FROM noun_type
    JOIN change_set ON change_set.change_set_id = noun_type.last_change_set_id;