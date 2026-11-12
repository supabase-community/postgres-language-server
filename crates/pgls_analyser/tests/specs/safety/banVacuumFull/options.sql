-- expect_lint/banVacuumFull
vacuum (full, analyze) my_table;
-- The last option wins.
-- expect_lint/banVacuumFull
vacuum (full false, full true) my_table;
vacuum (full, full false) my_table;
vacuum (full off) my_table;
-- Postgres rejects these values and names.
vacuum (full '0') my_table;
vacuum (full 2) my_table;
vacuum ("FULL") my_table;
