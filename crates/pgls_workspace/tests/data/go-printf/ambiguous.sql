-- a directive after a name can't be read, like the alias `%s` without `as`: the statement stays
-- as written. Found by the fuzzer: to Go, the `%t` of `FROM%tenk1` is a directive, so the
-- name `FROM%tenk1` follows `four`, and the formatter wrote `cast(FROM%tenk1 as four)`
select   a %s from t;
select ten, four FROM%tenk1 where unique2 < 10;
select   a as %s from t;
