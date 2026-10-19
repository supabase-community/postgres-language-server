-- sectionMarkers: dbmate
-- The down section runs after the up section, and the next up section after the down section
-- was undone.
-- migrate:up
CREATE TABLE posts (id bigint, title text);
CREATE TABLE tags (id bigint);

-- migrate:down
SELECT title FROM posts;
DROP TABLE tags;
DROP TABLE posts;

-- migrate:up
SELECT id FROM tags;
-- expect_lint/unknownRelation
SELECT id FROM comments;

-- migrate:down
SELECT 1;
