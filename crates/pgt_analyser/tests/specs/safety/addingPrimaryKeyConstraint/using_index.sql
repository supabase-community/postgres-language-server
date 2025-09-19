-- expect_no_diagnostics
-- This should not trigger the rule - using an existing index
ALTER TABLE items ADD CONSTRAINT items_pk PRIMARY KEY USING INDEX items_pk;