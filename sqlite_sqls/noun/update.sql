UPDATE noun
SET name = ?1,
    last_change_set_id = ?2,
    noun_type_id = (
        SELECT noun_type_id
        FROM noun_type
        where noun_type = ?3
    ),
    metadata = ?4
WHERE noun_id = ?5;