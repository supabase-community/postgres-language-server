select rank('adam'::varchar) within group (order by x) from (values ('fred'),('jim')) v(x);
