copy copytest to :'filename' csv;
copy copytest2 from :'filename' csv;
copy copytest to :'filename' csv quote '''' escape E'\\';
copy copytest2 from :'filename' csv quote '''' escape E'\\';
copy copytest3 from stdin csv header;
this is just a line full of junk that would error out if parsed
1,a,1
2,b,2
\.
copy copytest3 to stdout csv header;
copy copytest4 from stdin (header);
this is just a line full of junk that would error out if parsed
1	a
2	b
\.
copy copytest4 to stdout (header);
copy (select * from parted_copytest order by a) to :'filename';
copy parted_copytest from :'filename';
copy parted_copytest from :'filename' (freeze);
copy parted_copytest from :'filename';
copy parted_copytest from stdin;
1	1	str1
2	2	str2
\.
copy tab_progress_reporting from stdin;
sharon	25	(15,12)	1000	sam
sam	30	(10,5)	2000	bill
bill	20	(11,10)	1000	sharon
\.
copy tab_progress_reporting from :'filename'
	where (salary < 2000);
copy header_copytest to stdout with (header match);
copy header_copytest from stdin with (header wrong_choice);
copy header_copytest from stdin with (header match);
a	b	c
1	2	foo
\.
copy header_copytest (c, a, b) from stdin with (header match);
c	a	b
bar	3	4
\.
copy header_copytest from stdin with (header match, format csv);
a,b,c
5,6,baz
\.
copy header_copytest (c, b, a) from stdin with (header match);
a	b	c
1	2	foo
\.
copy header_copytest from stdin with (header match);
a	b	\N
1	2	foo
\.
copy header_copytest from stdin with (header match);
a	b
1	2
\.
copy header_copytest from stdin with (header match);
a	b	c	d
1	2	foo	bar
\.
copy header_copytest from stdin with (header match);
a	b	d
1	2	foo
\.
copy header_copytest (c, a) from stdin with (header match);
c	a
foo	7
\.
copy header_copytest (a, c) from stdin with (header match);
a	c
8	foo
\.
copy header_copytest from stdin with (header match);
a	........pg.dropped.2........	c
1	2	foo
\.
copy header_copytest (a, c) from stdin with (header match);
a	c	b
1	foo	2
\.
copy oversized_column_default from stdin;
\.
copy oversized_column_default (col2) from stdin;
\.
copy oversized_column_default from stdin (default '');
\.
COPY parted_si(id, data) FROM :'filename';