create function convslot_trig4() returns trigger as
$$begin raise exception 'BOOM!'; end$$ language plpgsql;
