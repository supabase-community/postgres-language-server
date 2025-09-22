create function outparam2_succeed(r anyrange, out lu anyarray, out ul anyarray)
  as $$ select array[lower($1), upper($1)], array[upper($1), lower($1)] $$
  language sql;
