SELECT noun_type_id,
    change_set.change_date,
    diff_noun_type,
    diff_metadata
FROM noun_type_history
    JOIN change_set on change_set.change_set_id = noun_type_history.change_set_id
where noun_type_history.ROWID = ?1;