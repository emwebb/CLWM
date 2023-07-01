SELECT data_type_name,
    system_defined,
    definition,
    version,
    change_set.change_date
FROM data_type
    JOIN change_set ON change_set.change_set_id = data_type.change_set_id
WHERE data_type.data_type_name = ?1
AND version = (SELECT MAX(version) FROM data_type WHERE data_type.data_type_name = ?1);