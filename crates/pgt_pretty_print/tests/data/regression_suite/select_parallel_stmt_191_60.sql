select count(*) from tenk1 a where (unique1, two) in
    (select unique1, row_number() over() from tenk1 b);
