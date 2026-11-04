-- myschema.varchar is a type of the user.
-- expect_lint/preferTextField
-- expect_lint/preferTextField
-- expect_lint/preferTextField
create table names (
    a varchar(255)[],
    b character varying(20) collate "C",
    c varchar /* name */ (40),
    d myschema.varchar(10)
);
