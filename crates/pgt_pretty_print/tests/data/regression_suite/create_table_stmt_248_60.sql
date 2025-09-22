create table parted_boolean_less partition of parted_boolean_col
  for values in ('foo' < 'bar');
