select range_agg(nmr) from (values ('{}'::nummultirange), ('{}'::nummultirange)) t(nmr);
