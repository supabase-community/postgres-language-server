create or replace function trigger_notice() returns trigger as $$
  begin
    raise notice 'trigger % on % % % for %', TG_NAME, TG_TABLE_NAME, TG_WHEN, TG_OP, TG_LEVEL;
    if TG_LEVEL = 'ROW' then
      if TG_OP = 'DELETE' then
        return OLD;
      else
        return NEW;
      end if;
    end if;
    return null;
  end;
  $$ language plpgsql;
