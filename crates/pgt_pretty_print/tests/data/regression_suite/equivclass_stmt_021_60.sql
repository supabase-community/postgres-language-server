create function int8alias1eq(int8alias1, int8alias2) returns bool
  strict immutable language internal as 'int8eq';
