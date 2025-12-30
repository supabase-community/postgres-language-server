MERGE INTO target_table AS t
USING source_table AS s ON t.id = s.id
WHEN MATCHED THEN UPDATE SET value = s.value
WHEN NOT MATCHED THEN INSERT (id, value) VALUES (s.id, s.value);