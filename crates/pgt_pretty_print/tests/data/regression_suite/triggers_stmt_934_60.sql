create function convslot_trig3()
returns trigger
language plpgsql
AS $$
begin
raise notice 'trigger = %, old_table = %, new table = %',
          TG_NAME,
          (select string_agg(old_table::text, ', ' order by col1) from old_table),
          (select string_agg(new_table::text, ', ' order by col1) from new_table);
return null;
end; $$;
