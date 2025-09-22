select * from int8_tbl i, lateral(values(i.*::int8_tbl)) ss;
