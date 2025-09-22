SELECT amname, amhandler, amtype FROM pg_am where amtype = 't' ORDER BY 1, 2;
