select * from %s.%s where id = %d and name = %q;
insert into %s (a, b) values (%d, %v);
update %s set a = %d where b in (%v, %v);
select a as %s, cast(b as %s) from t join %s on true;
