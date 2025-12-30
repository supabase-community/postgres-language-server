SELECT ''::text AS two, unique1, unique2, stringu1
		FROM onek WHERE unique1 > 50
		ORDER BY unique1 LIMIT 2;

SELECT ''::text AS five, unique1, unique2, stringu1
		FROM onek WHERE unique1 > 60
		ORDER BY unique1 LIMIT 5;

SELECT ''::text AS two, unique1, unique2, stringu1
		FROM onek WHERE unique1 > 60 AND unique1 < 63
		ORDER BY unique1 LIMIT 5;

SELECT ''::text AS three, unique1, unique2, stringu1
		FROM onek WHERE unique1 > 100
		ORDER BY unique1 LIMIT 3 OFFSET 20;

SELECT ''::text AS zero, unique1, unique2, stringu1
		FROM onek WHERE unique1 < 50
		ORDER BY unique1 DESC LIMIT 8 OFFSET 99;

SELECT ''::text AS eleven, unique1, unique2, stringu1
		FROM onek WHERE unique1 < 50
		ORDER BY unique1 DESC LIMIT 20 OFFSET 39;

SELECT ''::text AS ten, unique1, unique2, stringu1
		FROM onek
		ORDER BY unique1 OFFSET 990;

SELECT ''::text AS five, unique1, unique2, stringu1
		FROM onek
		ORDER BY unique1 OFFSET 990 LIMIT 5;

SELECT ''::text AS five, unique1, unique2, stringu1
		FROM onek
		ORDER BY unique1 LIMIT 5 OFFSET 900;

select * from int8_tbl limit (case when random() < 0.5 then null::bigint end);

select * from int8_tbl offset (case when random() < 0.5 then null::bigint end);

begin;

declare c1 cursor for select * from int8_tbl limit 10;

fetch all in c1;

fetch 1 in c1;

fetch backward 1 in c1;

fetch backward all in c1;

fetch backward 1 in c1;

fetch all in c1;

declare c2 cursor for select * from int8_tbl limit 3;

fetch all in c2;

fetch 1 in c2;

fetch backward 1 in c2;

fetch backward all in c2;

fetch backward 1 in c2;

fetch all in c2;

declare c3 cursor for select * from int8_tbl offset 3;

fetch all in c3;

fetch 1 in c3;

fetch backward 1 in c3;

fetch backward all in c3;

fetch backward 1 in c3;

fetch all in c3;

declare c4 cursor for select * from int8_tbl offset 10;

fetch all in c4;

fetch 1 in c4;

fetch backward 1 in c4;

fetch backward all in c4;

fetch backward 1 in c4;

fetch all in c4;
