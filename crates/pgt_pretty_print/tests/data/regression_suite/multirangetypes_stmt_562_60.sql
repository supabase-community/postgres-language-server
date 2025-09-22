create function anycompatiblearray_anycompatiblemultirange_func(a anycompatiblearray, mr anycompatiblemultirange)
  returns anycompatible as 'select $1[1] + lower($2);' language sql;
