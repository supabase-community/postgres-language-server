INSERT INTO base_tbl SELECT i, 'Row ' || i FROM generate_series(-2, 2) g(i);
