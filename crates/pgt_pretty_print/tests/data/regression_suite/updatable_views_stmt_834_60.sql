select table_name, is_updatable, is_insertable_into
  from information_schema.views where table_name = 'uv_ptv';
