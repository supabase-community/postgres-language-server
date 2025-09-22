SELECT * FROM JSON_TABLE(jsonb '"world"', '$' COLUMNS (item text PATH '$' WITH WRAPPER OMIT QUOTES));
