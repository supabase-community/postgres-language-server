CREATE EVENT TRIGGER regress_reindex_end ON ddl_command_end
    WHEN TAG IN ('REINDEX')
    EXECUTE PROCEDURE reindex_end_command();
