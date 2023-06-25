SELECT noun_id,
    name,
    change_set.change_date,
    noun_type,
    noun.metadata
FROM noun
    JOIN noun_type ON noun_type.noun_type_id = noun.noun_type_id
    JOIN change_set ON change_set.change_set_id = noun.last_change_set_id
WHERE noun.ROWID = ?1;