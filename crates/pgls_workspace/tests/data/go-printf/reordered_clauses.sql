-- the printer writes `limit` before `offset`, which would swap the arguments of Go: the
-- statements stay as written
select * from t offset %d limit %d;
select * from t offset '%d' limit '%d';
-- a comment, which Go reads too, moves with its clause and keeps its place before `limit %d`
select * from t offset 1 /* %d */ limit %d;
-- in the printer's order, it is formatted
select   * from t limit %d offset %d;
-- the printer writes the options of a function in its own order, before `as`: the comment after
-- `immutable` would move before the body, whose `%v` would take the comment's argument
create function f() returns int language sql as 'select %v' immutable /* %v */ strict;
-- the comment of `h` stays before the body, and the statement is formatted
create function h() /* %v */ returns int language sql strict as 'select %d' immutable;
