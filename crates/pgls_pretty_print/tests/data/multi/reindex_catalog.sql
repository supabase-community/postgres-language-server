REINDEX TABLE pg_class;

REINDEX TABLE pg_index;

REINDEX TABLE pg_operator;

REINDEX TABLE pg_database;

REINDEX TABLE pg_shdescription;

REINDEX INDEX pg_class_oid_index;

REINDEX INDEX pg_class_relname_nsp_index;

REINDEX INDEX pg_index_indexrelid_index;

REINDEX INDEX pg_index_indrelid_index;

REINDEX INDEX pg_database_oid_index;

REINDEX INDEX pg_shdescription_o_c_index;

BEGIN;

SET min_parallel_table_scan_size = 0;

REINDEX INDEX pg_class_oid_index;

REINDEX INDEX pg_class_relname_nsp_index;

REINDEX INDEX pg_index_indexrelid_index;

REINDEX INDEX pg_index_indrelid_index;

REINDEX INDEX pg_database_oid_index;

REINDEX INDEX pg_shdescription_o_c_index;

ROLLBACK;
