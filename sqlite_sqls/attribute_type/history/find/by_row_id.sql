SELECT attribute_type_id,
    change_set.change_date,
    diff_attribute_name,
    diff_multiple_allowed,
    diff_metadata
from attribute_type_history
    JOIN change_set ON change_set.change_set_id = attribute_type_history.change_set_id
WHERE attribute_type_history.ROWID = ?;