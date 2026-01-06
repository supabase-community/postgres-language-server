CREATE TABLE delete_test (
    id SERIAL PRIMARY KEY,
    a INT,
    b text
);

INSERT INTO delete_test (a) VALUES (10);

INSERT INTO delete_test (a, b) VALUES (50, repeat('x', 10000));

INSERT INTO delete_test (a) VALUES (100);

DELETE FROM delete_test AS dt WHERE dt.a > 75;

DELETE FROM delete_test dt WHERE delete_test.a > 25;

SELECT id, a, char_length(b) FROM delete_test;

DELETE FROM delete_test WHERE a > 25;

SELECT id, a, char_length(b) FROM delete_test;

DROP TABLE delete_test;
