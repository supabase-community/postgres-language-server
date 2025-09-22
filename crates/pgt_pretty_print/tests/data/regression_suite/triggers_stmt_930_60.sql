create function convslot_trig1()
returns trigger
language plpgsql
AS $$
begin
raise notice 'trigger = %, old_table = %',
          TG_NAME,
          (select string_agg(old_table::text, ', ' order by col1) from old_table);
return null;
end; $$;
