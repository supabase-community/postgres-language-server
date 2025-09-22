create function fcompos2(v compos) returns void as $$
select fcompos1(v);
$$ language sql;
