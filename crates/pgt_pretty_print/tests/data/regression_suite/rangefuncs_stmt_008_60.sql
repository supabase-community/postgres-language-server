select a,ord from unnest(array['a','b']) with ordinality as z(a,ord);
