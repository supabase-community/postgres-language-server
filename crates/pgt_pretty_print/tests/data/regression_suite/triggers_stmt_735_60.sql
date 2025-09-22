create or replace function dump_update_new() returns trigger language plpgsql as
$$
  begin
    raise notice 'trigger = %, new table = %', TG_NAME,
                 (select string_agg(new_table::text, ', ' order by a) from new_table);
    return null;
  end;
$$;
