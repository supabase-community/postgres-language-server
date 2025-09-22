SELECT x,
       gamma(x),
       lgamma(x)
FROM (VALUES (0.5), (1), (2), (3), (4), (5),
             (float8 'infinity'), (float8 'nan')) AS t(x);
