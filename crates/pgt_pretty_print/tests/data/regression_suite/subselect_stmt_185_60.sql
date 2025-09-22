select * from outer_text where (f1, f2) not in (select * from inner_text);
