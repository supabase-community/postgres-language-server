INSERT INTO users (id, name) VALUES (1, 'John') ON CONFLICT (id) DO NOTHING;
