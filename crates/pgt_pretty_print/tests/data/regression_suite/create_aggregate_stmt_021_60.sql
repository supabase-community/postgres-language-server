create function least_accum(anycompatible, anycompatible)
returns anycompatible language sql as
  'select least($1, $2)';
