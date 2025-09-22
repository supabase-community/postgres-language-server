create unique index both_index_expr_key on insertconflicttest(key, lower(fruit) collate "C" text_pattern_ops);
