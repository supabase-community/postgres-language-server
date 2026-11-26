create procedure note_item(id int8, note text) language sql as 'select 1';
create procedure note_item(id numeric, note text) language sql as 'select 1';
create function add_one(x int) returns int language sql as 'select x + 1';
-- A CALL taints the catalog, so only the first one of a file is checked.
-- expect_lint/functionArgumentMismatch
call note_item(1, 'x');
