SELECT * FROM dropped_objects
  WHERE schema_name IS NULL OR schema_name <> 'pg_toast';
