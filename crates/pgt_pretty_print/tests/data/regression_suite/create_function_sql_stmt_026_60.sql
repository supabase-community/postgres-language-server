SELECT proname, prosecdef FROM pg_proc
       WHERE oid in ('functest_C_1'::regproc,
                     'functest_C_2'::regproc,
                     'functest_C_3'::regproc) ORDER BY proname;
