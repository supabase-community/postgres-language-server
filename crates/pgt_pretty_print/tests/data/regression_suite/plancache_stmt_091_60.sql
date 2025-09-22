select name, generic_plans, custom_plans from pg_prepared_statements
  where  name = 'test_mode_pp';
