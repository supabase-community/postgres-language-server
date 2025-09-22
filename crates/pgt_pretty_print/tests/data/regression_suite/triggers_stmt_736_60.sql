create or replace function dump_update_old() returns trigger language plpgsql as
$$
  begin
    raise notice 'trigger = %, old table = %', TG_NAME,
                 (select string_agg(old_table::text, ', ' order by a) from old_table);
    return null;
  end;
$$;
