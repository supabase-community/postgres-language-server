-- Binary coercible changes, and changes that keep the type, don't rewrite the table.
-- catalog: builtin
-- expect_no_diagnostics
create table recipes (id int8, name varchar(20), body text, network cidr);
alter table recipes alter column name type text;
alter table recipes alter column body type varchar;
alter table recipes alter column id type bigint;
alter table recipes alter column network type inet;
alter table recipes alter column name type text using name;
