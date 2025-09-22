select date_bin('365000 days'::interval, '4400-01-01 BC'::timestamptz, '4000-01-01 BC'::timestamptz);
