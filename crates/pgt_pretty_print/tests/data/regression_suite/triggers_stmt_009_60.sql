create function f1_times_10() returns trigger as
$$ begin new.f1 := new.f1 * 10; return new; end $$ language plpgsql;
