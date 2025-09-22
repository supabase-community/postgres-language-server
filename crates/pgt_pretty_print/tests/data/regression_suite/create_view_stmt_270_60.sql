select * from int8_tbl i where i.* in (values(i.*::int8_tbl));
