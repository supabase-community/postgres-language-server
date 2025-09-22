create function convslot_trig2()
returns trigger
language plpgsql
AS $$
begin
raise notice 'trigger = %, new table = %',
          TG_NAME,
          (select string_agg(new_table::text, ', ' order by col1) from new_table);
return null;
end; $$;
