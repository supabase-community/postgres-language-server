create or replace function parted_conflict_update_func() returns trigger as $$
declare
    r record;
begin
 for r in select * from inserted loop
	raise notice 'a = %, b = %, c = %', r.a, r.b, r.c;
 end loop;
 return new;
end;
$$ language plpgsql;
