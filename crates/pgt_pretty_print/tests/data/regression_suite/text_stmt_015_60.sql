select i, left('ahoj', i), right('ahoj', i) from generate_series(-5, 5) t(i) order by i;
