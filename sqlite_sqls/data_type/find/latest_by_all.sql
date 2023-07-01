SELECT data_type.data_type_name,
    data_type.system_defined,
    data_type.definition,
    data_type.version,
    change_set.change_date
FROM data_type
    JOIN change_set ON change_set.change_set_id = data_type.change_set_id
INNER JOIN (
    SELECT data_type_name,
        MAX(version) AS max_version
    FROM data_type
    GROUP BY data_type_name
) AS latest ON latest.data_type_name = data_type.data_type_name
AND latest.max_version = data_type.version;