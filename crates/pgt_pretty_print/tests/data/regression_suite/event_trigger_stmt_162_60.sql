CREATE EVENT TRIGGER regress_reindex_start ON ddl_command_start
    WHEN TAG IN ('REINDEX')
    EXECUTE PROCEDURE reindex_start_command();
