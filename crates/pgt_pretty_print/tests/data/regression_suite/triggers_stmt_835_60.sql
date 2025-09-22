create function make_bogus_matview() returns trigger as
$$ begin
  create materialized view transition_test_mv as select * from new_table;
  return new;
end $$
language plpgsql;
