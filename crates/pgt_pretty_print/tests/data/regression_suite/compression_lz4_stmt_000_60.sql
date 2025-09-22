SELECT NOT(enumvals @> '{lz4}') AS skip_test FROM pg_settings WHERE
  name = 'default_toast_compression' ;
