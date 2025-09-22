CREATE TABLE gtest_child2 PARTITION OF gtest_parent (
    f3 WITH OPTIONS GENERATED ALWAYS AS (f2 * 22) STORED  -- overrides gen expr
) FOR VALUES FROM ('2016-08-01') TO ('2016-09-01');
