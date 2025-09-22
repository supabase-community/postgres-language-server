create function bogus_func(anycompatible)
  returns anycompatiblerange as 'select int4range(1,10)' language sql;
