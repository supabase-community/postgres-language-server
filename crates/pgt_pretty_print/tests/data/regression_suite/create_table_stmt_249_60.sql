create table parted_boolean_greater partition of parted_boolean_col
  for values in ('foo' > 'bar');
