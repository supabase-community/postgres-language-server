SELECT r0.routine_name, r1.routine_name
  FROM information_schema.routine_routine_usage rru
       JOIN information_schema.routines r0 ON r0.specific_name = rru.specific_name
       JOIN information_schema.routines r1 ON r1.specific_name = rru.routine_name
  WHERE r0.routine_schema = 'temp_func_test' AND
        r1.routine_schema = 'temp_func_test'
  ORDER BY 1, 2;
