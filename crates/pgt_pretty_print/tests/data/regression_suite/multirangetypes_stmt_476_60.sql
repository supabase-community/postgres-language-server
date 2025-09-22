select range_intersect_agg(nmr) from (values ('{[1,2]}'::nummultirange)) t(nmr);
