insert into t (a) values (1) returning with (old as o, new as n) o.a, n.*;
insert into t (a) values (1) returning old.a, new.*;
update t set a = 1 returning old.*, new.a;
update t set a = 1 returning with (old as previous) previous.a, new.a;
analyze only t;
analyze only t, only s;
