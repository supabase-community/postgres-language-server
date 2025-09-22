SELECT pg_size_pretty('-9223372036854775808'::bigint),
       pg_size_pretty('9223372036854775807'::bigint);
