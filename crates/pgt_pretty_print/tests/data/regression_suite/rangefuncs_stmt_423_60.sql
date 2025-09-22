select * from
  (select jsonb_path_query_array(module->'lectures', '$[*]') as lecture
   from unnest(array['{"lectures": [{"id": "1"}]}'::jsonb])
        as unnested_modules(module)) as ss,
  jsonb_to_recordset(ss.lecture) as j (id text);
