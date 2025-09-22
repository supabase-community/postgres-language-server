create function table_succeed(r anyrange)
  returns table(l anyelement, u anyelement)
  as $$ select lower($1), upper($1) $$
  language sql;
