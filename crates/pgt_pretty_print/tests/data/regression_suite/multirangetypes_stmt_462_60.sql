select range_agg(nmr) from (values ('{}'::nummultirange)) t(nmr);
