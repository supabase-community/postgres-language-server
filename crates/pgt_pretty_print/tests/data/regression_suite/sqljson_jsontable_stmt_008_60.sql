SELECT * FROM JSON_TABLE(jsonb '{"rec": "(1,2)"}', '$' COLUMNS (id FOR ORDINALITY, comp comp path '$.rec' omit quotes)) jt;
