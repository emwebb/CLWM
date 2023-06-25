UPDATE noun_type
SET noun_type = ?1,
    last_change_set_id = ?2,
    metadata = ?3
WHERE noun_type_id = ?4;