create function qqq_trig() returns trigger as $$
begin
if tg_op in ('INSERT', 'UPDATE') then
    raise notice '% % %', tg_when, tg_op, new.id;
    return new;
else
    raise notice '% % %', tg_when, tg_op, old.id;
    return old;
end if;
end
$$ language plpgsql;
