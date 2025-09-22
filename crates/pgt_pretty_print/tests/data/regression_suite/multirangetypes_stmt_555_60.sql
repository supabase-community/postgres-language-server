create function bogus_func(int)
  returns anymultirange as 'select int4multirange(int4range(1,10))' language sql;
