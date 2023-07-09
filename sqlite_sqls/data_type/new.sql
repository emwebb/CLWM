INSERT INTO data_type (data_type_name)
VALUES (?);
INSERT INTO data_type_version (
        data_type_name,
        system_defined,
        definition,
        version,
        change_set_id
    )
VALUES (?, ?, ?, ?, ?);