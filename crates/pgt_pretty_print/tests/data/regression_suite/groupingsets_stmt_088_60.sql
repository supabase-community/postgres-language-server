select count(*) from gstest4 group by rollup(unhashable_col,unsortable_col);
