  select
    view_id, view_schema, view_name,
    json_array_elements(view_definition->0->'targetList') as entry
  from transform_json
