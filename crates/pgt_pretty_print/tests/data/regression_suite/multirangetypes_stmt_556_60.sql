create function range_add_bounds(anymultirange)
  returns anyelement as 'select lower($1) + upper($1)' language sql;
