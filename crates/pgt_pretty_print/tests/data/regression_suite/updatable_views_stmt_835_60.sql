select table_name, column_name, is_updatable
  from information_schema.columns where table_name = 'uv_ptv' order by column_name;
