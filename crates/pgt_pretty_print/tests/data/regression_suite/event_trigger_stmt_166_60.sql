CREATE EVENT TRIGGER regress_reindex_end_snap ON ddl_command_end
    EXECUTE FUNCTION reindex_end_command_snap();
