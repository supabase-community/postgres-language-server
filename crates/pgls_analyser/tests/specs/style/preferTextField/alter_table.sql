-- expect_lint/preferTextField
alter table users alter column name type varchar(100) collate "C";
