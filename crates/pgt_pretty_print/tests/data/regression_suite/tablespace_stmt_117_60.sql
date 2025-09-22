ALTER TABLE testschema.test_default_tab ADD CONSTRAINT test_index4 UNIQUE (id) USING INDEX TABLESPACE regress_tblspace;
