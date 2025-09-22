with testdata(bits) as (values
  -- small subnormals
  (x'0000000000000001'),
  (x'0000000000000002'), (x'0000000000000003'),
  (x'0000000000001000'), (x'0000000100000000'),
  (x'0000010000000000'), (x'0000010100000000'),
  (x'0000400000000000'), (x'0000400100000000'),
  (x'0000800000000000'), (x'0000800000000001'),
  -- these values taken from upstream testsuite
  (x'00000000000f4240'),
  (x'00000000016e3600'),
  (x'0000008cdcdea440'),
  -- borderline between subnormal and normal
  (x'000ffffffffffff0'), (x'000ffffffffffff1'),
  (x'000ffffffffffffe'), (x'000fffffffffffff'))
select float8send(flt) as ibits,
       flt
  from (select bits::bigint::xfloat8::float8 as flt
          from testdata
	offset 0) s;
