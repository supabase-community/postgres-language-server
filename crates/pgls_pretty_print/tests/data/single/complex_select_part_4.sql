  select
    view_id, view_schema, view_name,
    (entry->>'resno')::int as view_column,
    (entry->>'resorigtbl')::oid as resorigtbl,
    (entry->>'resorigcol')::int as resorigcol
  from target_entries
