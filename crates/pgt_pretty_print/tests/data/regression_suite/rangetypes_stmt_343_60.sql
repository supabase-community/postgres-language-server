create function range_add_bounds(anyrange)
  returns anyelement as 'select lower($1) + upper($1)' language sql;
