create aggregate least_agg(int8) (
  stype = int8, sfunc = least_accum
);
