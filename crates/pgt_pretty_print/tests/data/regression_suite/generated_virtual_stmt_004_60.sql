SELECT table_name, column_name, dependent_column FROM information_schema.column_column_usage WHERE table_schema = 'generated_virtual_tests' ORDER BY 1, 2, 3;
