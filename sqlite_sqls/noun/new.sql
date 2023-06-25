INSERT INTO noun (name, last_change_set_id, noun_type_id, metadata)
VALUES (
        ?1,
        ?2,
        (
            SELECT noun_type_id
            FROM noun_type
            where noun_type = ?3
        ),
        ?4
    );