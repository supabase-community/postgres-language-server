INSERT INTO guid3 (guid_field) SELECT uuidv7() FROM generate_series(1, 10);
