select range_agg(nmr) from (values ('{[1,2], [5,6]}'::nummultirange)) t(nmr);
