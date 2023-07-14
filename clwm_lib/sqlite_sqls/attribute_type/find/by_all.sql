SELECT attribute_type.attribute_type_id,
    attribute_name,
    data_type_name,
    multiple_allowed,
    metadata,
    change_set.change_date
from attribute_type
    JOIN change_set ON change_set.change_set_id = attribute_type.last_change_set_id;