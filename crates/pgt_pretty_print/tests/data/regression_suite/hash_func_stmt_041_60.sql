SELECT hashfloat8('NaN'::float8) = hashfloat8(-'NaN'::float8) AS t;
