select * from float_table
  where float_col in (select num_col from numeric_table);
