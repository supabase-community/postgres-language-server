PREPARE my_insert (int, text) AS
INSERT INTO users (id, name) VALUES ($1, $2);