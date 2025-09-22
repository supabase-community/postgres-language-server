create function sillysrf(int) returns setof int as
  'values (1),(10),(2),($1)' language sql immutable;
