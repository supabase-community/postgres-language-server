select * from numeric_table
  where num_col in (select float_col from float_table);
