SELECT routine_name, table_name FROM information_schema.routine_table_usage
  WHERE routine_schema = 'temp_func_test'
  ORDER BY 1, 2;
