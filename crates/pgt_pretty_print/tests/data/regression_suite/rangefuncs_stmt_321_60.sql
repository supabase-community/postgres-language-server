create function testrngfunc() returns record as $$
  insert into rngfunc values (1,2) returning *;
$$ language sql;
