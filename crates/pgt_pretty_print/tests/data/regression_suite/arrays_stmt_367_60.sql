select v, v is null as "is null" from string_to_table('abc', ',', 'abc') g(v);
