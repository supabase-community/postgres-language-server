select unnest(textmultirange(textrange('a', 'b'), textrange('d', 'e')));
