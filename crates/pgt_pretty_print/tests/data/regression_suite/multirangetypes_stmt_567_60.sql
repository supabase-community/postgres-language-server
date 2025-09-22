create function anycompatiblerange_anycompatiblemultirange_func(r anycompatiblerange, mr anycompatiblemultirange)
  returns anycompatible as 'select lower($1) + lower($2);' language sql;
