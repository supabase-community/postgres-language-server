SELECT '{"a":"c"}'::jsonb <@ '{"a":"b", "b":1, "c":null}';
