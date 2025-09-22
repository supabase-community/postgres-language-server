create function least_accum(int8, int8) returns int8 language sql as
  'select least($1, $2)';
