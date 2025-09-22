SELECT table_name, column_name, column_default, is_nullable, is_generated, generation_expression FROM information_schema.columns WHERE table_schema = 'generated_stored_tests' ORDER BY 1, 2;
