-- A query that doesn't parse can't create anything, so later statements are still checked.
-- expect_lint/unknownRelation
select * frm somewhere;
select * from missing_table;
