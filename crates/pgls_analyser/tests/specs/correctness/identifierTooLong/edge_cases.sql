-- 32 two-byte characters are 64 bytes.
-- expect_lint/identifierTooLong
create table ääääääääääääääääääääääääääääääää (id int);
-- 31 two-byte characters are 62 bytes, and the doubled quote stands for one byte: 63 bytes.
create table äääääääääääääääääääääääääääääää (id int);
create table "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""" (id int);
-- Strings, comments and dollar-quoted bodies are not identifiers.
select 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' /* aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa */;
create function f() returns int language plpgsql as $$ begin return 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'; end $$;
-- Quotes in strings and comments don't hide a long quoted identifier.
-- expect_lint/identifierTooLong
select '"' as a, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa b c d" /* " */;
-- A name used several times in a statement is reported once. Unquoted names are downcased.
-- expect_lint/identifierTooLong
select aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.id from aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa where AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA.id = 1;
