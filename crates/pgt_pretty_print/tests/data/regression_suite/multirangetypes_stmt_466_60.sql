select range_agg(nmr) from (values ('{[1,2], [2,3]}'::nummultirange)) t(nmr);
