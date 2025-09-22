select any_value(v) filter (where v > 2) from (values (1), (2), (3)) as v (v);
