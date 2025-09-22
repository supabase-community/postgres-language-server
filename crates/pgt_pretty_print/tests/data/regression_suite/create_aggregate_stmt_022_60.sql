create aggregate least_agg(int4) (
  stype = int8, sfunc = least_accum
);
