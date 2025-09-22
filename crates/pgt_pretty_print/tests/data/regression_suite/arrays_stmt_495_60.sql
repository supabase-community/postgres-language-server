SELECT array_sort(ARRAY(SELECT '1 4'::int2vector UNION ALL SELECT '1 2'::int2vector));
