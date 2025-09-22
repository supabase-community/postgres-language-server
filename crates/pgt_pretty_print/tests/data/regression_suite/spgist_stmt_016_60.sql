insert into spgist_text_tbl (id, t)
select g, 'f' || repeat('o', 100) || g from generate_series(1, 10000) g
union all
select g, 'baaaaaaaaaaaaaar' || g from generate_series(1, 1000) g;
