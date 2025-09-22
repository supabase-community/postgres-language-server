SELECT routine_name, table_name, column_name FROM information_schema.routine_column_usage
  WHERE routine_schema = 'temp_func_test'
  ORDER BY 1, 2;
