-- section: up
CREATE TABLE posts (id bigint, title text);
CREATE INDEX posts_title_idx ON posts (title);

-- In the down section, the table isn't new anymore.
-- section: down
-- expect_lint/requireConcurrentIndexCreation
CREATE INDEX posts_id_idx ON posts (id);
DROP TABLE posts;
