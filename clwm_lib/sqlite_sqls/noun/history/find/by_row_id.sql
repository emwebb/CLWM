SELECT noun_id,
    change_set.change_date,
    diff_name,
    diff_noun_type,
    diff_metadata
FROM noun_history
    JOIN change_set on change_set.change_set_id = noun_history.change_set_id
where noun_history.ROWID = ?1;