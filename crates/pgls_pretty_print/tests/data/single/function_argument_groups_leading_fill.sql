-- pgls-format: lineWidth=60, commaStyle=leading, functionArgumentGroups=my_triple_func:3
-- A group that doesn't fit on its line fills it, as with trailing commas, instead of putting
-- each of its arguments on a line of its own.
SELECT my_triple_func(
    'first_key', first_value_column_name, 'first_unit_label_text',
    'second_key', second_value_column_name, 'second_unit_label_text'
) FROM t;
