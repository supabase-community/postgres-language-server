create function f () returns trigger as
$$ begin return new; end; $$
language plpgsql;
