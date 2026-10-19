-- The down section runs after the up section, and the next up section after the down section
-- was undone.
-- section: up
CREATE TABLE posts (id bigint, title text);
CREATE TABLE tags (id bigint);

-- section: down
SELECT title FROM posts;
DROP TABLE tags;
DROP TABLE posts;

-- section: up
SELECT id FROM tags;
-- expect_lint/unknownRelation
SELECT id FROM comments;

-- section: down
SELECT 1;
