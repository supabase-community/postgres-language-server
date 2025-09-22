create function anycompatiblearray_anycompatiblerange_func(a anycompatiblearray, r anycompatiblerange)
  returns anycompatible as 'select $1[1] + lower($2);' language sql;
