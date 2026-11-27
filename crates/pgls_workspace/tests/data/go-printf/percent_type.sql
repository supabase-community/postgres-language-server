-- found by the fuzzer: to Go, the `%T` of `%TYPE` is a directive, and the body of the
-- function, which has none, is formatted
CREATE FUNCTION hobbies_by_name(hobbies_r.name%TYPE)
   RETURNS hobbies_r.person%TYPE
   AS 'select person from hobbies_r where name = $1'
   LANGUAGE SQL;
