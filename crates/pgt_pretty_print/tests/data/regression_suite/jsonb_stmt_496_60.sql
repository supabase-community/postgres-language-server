SELECT jsonb_populate_recordset(row(1,2)::jb_ordered_pair, '[{"x": 1, "y": 0}]');
