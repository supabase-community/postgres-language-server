select 'create table hp_prefix_test_p' || x::text || ' partition of hp_prefix_test for values with (modulus 8, remainder ' || x::text || ');'
from generate_Series(0,7) x;
