select (stringu1 || repeat('abcd', 5000))::int2 from tenk1 where unique1 = 1;
