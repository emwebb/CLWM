SELECT data_type_version.data_type_name,
    data_type_version.system_defined,
    data_type_version.definition,
    data_type_version.version,
    change_set.change_date
FROM data_type_version
    JOIN change_set ON change_set.change_set_id = data_type_version.change_set_id
    INNER JOIN (
        SELECT data_type_name,
            MAX(version) AS max_version
        FROM data_type_version
        GROUP BY data_type_name
    ) AS latest ON latest.data_type_name = data_type_version.data_type_name
    AND latest.max_version = data_type_version.version;