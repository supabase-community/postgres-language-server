select v, v is null as "is null" from string_to_table('1,2,3,4,*,6', ',', '*') g(v);
