select a,ord from unnest(array[1.0::float8]) with ordinality as z(a,ord);
