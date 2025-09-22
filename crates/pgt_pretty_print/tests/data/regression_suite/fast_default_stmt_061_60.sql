ALTER TABLE T ADD COLUMN c_array TEXT[]
                  DEFAULT ('{"This", "is", "' || foo(4) ||
                           '","the", "real", "world"}')::TEXT[],
              ALTER COLUMN c_timestamp
                  SET DEFAULT '1970-12-31'::DATE + LENGTH(foo(30));
