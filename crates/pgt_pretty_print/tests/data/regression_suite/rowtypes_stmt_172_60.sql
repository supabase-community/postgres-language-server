create function fcompos1(v compos) returns void as $$
insert into compos values (v.*);
$$ language sql;
