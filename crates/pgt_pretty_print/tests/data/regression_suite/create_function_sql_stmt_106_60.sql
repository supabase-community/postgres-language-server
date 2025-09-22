SELECT routine_name, sequence_name FROM information_schema.routine_sequence_usage
  WHERE routine_schema = 'temp_func_test'
  ORDER BY 1, 2;
