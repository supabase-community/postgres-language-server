create aggregate least_agg(variadic items anyarray) (
  stype = anyelement, sfunc = least_accum
);
