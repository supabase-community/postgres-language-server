select * from ec0 m join ec0 n on m.ff = n.ff
  join ec1 p on p.f1::int8 = (m.ff + n.ff)::int8alias1;
