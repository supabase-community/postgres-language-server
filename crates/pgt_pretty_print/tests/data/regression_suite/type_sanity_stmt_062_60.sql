SELECT oid, typname, typtype, typelem, typarray
  FROM pg_type t
  WHERE oid < 16384 AND
    -- Exclude pseudotypes and composite types.
    typtype NOT IN ('p', 'c') AND
    -- These reg* types cannot be pg_upgraded, so discard them.
    oid != ALL(ARRAY['regproc', 'regprocedure', 'regoper',
                     'regoperator', 'regconfig', 'regdictionary',
                     'regnamespace', 'regcollation']::regtype[]) AND
    -- Discard types that do not accept input values as these cannot be
    -- tested easily.
    -- Note: XML might be disabled at compile-time.
    oid != ALL(ARRAY['gtsvector', 'pg_node_tree',
                     'pg_ndistinct', 'pg_dependencies', 'pg_mcv_list',
                     'pg_brin_bloom_summary',
                     'pg_brin_minmax_multi_summary', 'xml']::regtype[]) AND
    -- Discard arrays.
    NOT EXISTS (SELECT 1 FROM pg_type u WHERE u.typarray = t.oid)
    -- Exclude everything from the table created above.  This checks
    -- that no in-core types are missing in tab_core_types.
    AND NOT EXISTS (SELECT 1
                    FROM pg_attribute a
                    WHERE a.atttypid=t.oid AND
                          a.attnum > 0 AND
                          a.attrelid='tab_core_types'::regclass);
