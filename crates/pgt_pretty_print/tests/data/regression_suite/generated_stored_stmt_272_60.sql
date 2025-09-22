CREATE TABLE gtest_child3 PARTITION OF gtest_parent (
    f3 DEFAULT 42  -- error
) FOR VALUES FROM ('2016-09-01') TO ('2016-10-01');
