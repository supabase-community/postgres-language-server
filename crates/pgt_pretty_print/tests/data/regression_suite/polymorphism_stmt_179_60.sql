create aggregate build_group(anyelement, integer) (
  SFUNC = add_group,
  STYPE = anyarray
);
