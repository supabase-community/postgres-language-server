CREATE FUNCTION test_trigger()
RETURNS trigger
LANGUAGE plpgsql
AS $$

begin
    raise notice 'old tuple: %', to_json(OLD)::text;
    if TG_OP = 'DELETE'
    then
       return OLD;
    else
       return NEW;
    end if;
end;

$$;
