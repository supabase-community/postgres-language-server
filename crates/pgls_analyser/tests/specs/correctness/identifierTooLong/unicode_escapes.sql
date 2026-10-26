-- Postgres decodes U&"..." identifiers and truncates them like any other identifier.
-- expect_lint/identifierTooLong
create table U&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" (id int);
-- expect_lint/identifierTooLong
create table U&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0061\0061\0061\0061" (id int);
-- expect_lint/identifierTooLong
create table U&"x!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061!0061" uescape '!' (id int);
-- A 4-byte character and 60 more bytes, written with 6 and with surrogate pair escapes.
-- expect_lint/identifierTooLong
create table u&"\+01F600aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" (id int);
-- expect_lint/identifierTooLong
create table U&"\D83D\DE00aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" (id int);
-- The escapes are longer than the 63 bytes they stand for.
create table U&"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!0061!0061!0061" UESCAPE '!' (id int);
create table U&"\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061\0061" (id int);
