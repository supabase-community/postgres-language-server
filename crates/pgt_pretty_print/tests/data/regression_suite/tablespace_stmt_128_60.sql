CREATE TABLE testschema.test_default_tab_p1 PARTITION OF testschema.test_default_tab_p
    FOR VALUES IN (1);
