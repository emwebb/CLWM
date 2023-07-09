SELECT data_type_name,
    system_defined,
    definition,
    version,
    change_set.change_date
FROM data_type_version
    JOIN change_set ON change_set.change_set_id = data_type_version.change_set_id;