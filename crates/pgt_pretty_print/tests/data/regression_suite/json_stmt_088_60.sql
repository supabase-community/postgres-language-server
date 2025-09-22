SELECT row_to_json(q)
FROM (SELECT 'NaN'::float8 AS "float8field") q;
