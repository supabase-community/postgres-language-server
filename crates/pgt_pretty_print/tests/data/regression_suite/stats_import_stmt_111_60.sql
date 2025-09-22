INSERT INTO stats_import.test
SELECT 1, 'one', (1, 1.1, 'ONE', '2001-01-01', '{ "xkey": "xval" }')::stats_import.complex_type, int4range(1,4), array['red','green']
UNION ALL
SELECT 2, 'two', (2, 2.2, 'TWO', '2002-02-02', '[true, 4, "six"]')::stats_import.complex_type,  int4range(1,4), array['blue','yellow']
UNION ALL
SELECT 3, 'tre', (3, 3.3, 'TRE', '2003-03-03', NULL)::stats_import.complex_type, int4range(-1,1), array['"orange"', 'purple', 'cyan']
UNION ALL
SELECT 4, 'four', NULL, int4range(0,100), NULL;
