CREATE TRIGGER alter_table_under_transition_tables_upd_trigger
  AFTER TRUNCATE OR UPDATE ON alter_table_under_transition_tables
  REFERENCING OLD TABLE AS d NEW TABLE AS i
  FOR EACH STATEMENT EXECUTE PROCEDURE
    alter_table_under_transition_tables_upd_func();
