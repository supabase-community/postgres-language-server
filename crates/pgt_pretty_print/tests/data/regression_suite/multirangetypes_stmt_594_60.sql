create function mr_outparam_succeed4(i anyrange, out r anymultirange, out t text)
  as $$ select multirange($1), 'foo'::text $$ language sql;
