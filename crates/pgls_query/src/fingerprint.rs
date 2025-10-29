use std::ffi::{CStr, CString};

use crate::bindings::*;
use crate::error::*;

/// Represents the resulting fingerprint containing both the raw integer form as well as the
/// corresponding 16 character hex value.
pub struct Fingerprint {
    pub value: u64,
    pub hex: String,
}

/// Fingerprints the given SQL statement. Useful for comparing parse trees across different implementations
/// of `libpg_query`.
///
/// # Example
///
/// ```rust
/// let result = pgls_query::fingerprint("SELECT * FROM contacts WHERE name='Paul'");
/// assert!(result.is_ok());
/// let result = result.unwrap();
/// assert_eq!(result.hex, "0e2581a461ece536");
/// ```
pub fn fingerprint(statement: &str) -> Result<Fingerprint> {
    let input = CString::new(statement)?;
    let result = unsafe { pg_query_fingerprint(input.as_ptr()) };
    let fingerprint = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Parse(message))
    } else {
        let hex = unsafe { CStr::from_ptr(result.fingerprint_str) };
        Ok(Fingerprint {
            value: result.fingerprint,
            hex: hex.to_string_lossy().to_string(),
        })
    };
    unsafe { pg_query_free_fingerprint_result(result) };
    fingerprint
}

#[cfg(test)]
mod tests {
    use crate::{Error, fingerprint};

    #[test]
    fn it_can_fingerprint_a_simple_statement() {
        let result =
            fingerprint("SELECT * FROM contacts.person WHERE id IN (1, 2, 3, 4);").unwrap();
        assert_eq!(result.hex, "643d2a3c294ab8a7");
    }

    #[test]
    fn it_will_error_on_invalid_input() {
        let error = fingerprint("CREATE RANDOM ix_test ON contacts.person;")
            .err()
            .unwrap();
        assert_eq!(
            error,
            Error::Parse("syntax error at or near \"RANDOM\"".into())
        );
    }

    #[test]
    fn it_works_for_multi_statement_queries() {
        let q1 = "SET x=$1; SELECT A";
        let q2 = "SET x=$1; SELECT a";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        let q1 = "SET x=$1; SELECT A";
        let q2 = "SELECT a";
        assert_ne!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
    }

    #[test]
    fn it_ignores_aliases() {
        let q1 = "SELECT a AS b";
        let q2 = "SELECT a AS c";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        let q1 = "SELECT a";
        let q2 = "SELECT a AS c";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        let q1 = "SELECT * FROM a AS b";
        let q2 = "SELECT * FROM a AS c";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        let q1 = "SELECT * FROM a";
        let q2 = "SELECT * FROM a AS c";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        let q1 = "SELECT * FROM (SELECT * FROM x AS y) AS a";
        let q2 = "SELECT * FROM (SELECT * FROM x AS z) AS b";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        let q1 = "SELECT a AS b UNION SELECT x AS y";
        let q2 = "SELECT a AS c UNION SELECT x AS z";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
    }

    #[test]
    fn it_ignores_param_references() {
        let q1 = "SELECT $1";
        let q2 = "SELECT $2";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
    }

    #[test]
    fn it_ignores_select_target_list_ordering() {
        let q1 = "SELECT a, b FROM x";
        let q2 = "SELECT b, a FROM x";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
        let q1 = "SELECT $1, b FROM x";
        let q2 = "SELECT b, $1 FROM x";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
        let q1 = "SELECT $1, $2, b FROM x";
        let q2 = "SELECT $1, b, $2 FROM x";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        // Testing uniqueness
        let q1 = "SELECT a, c FROM x";
        let q2 = "SELECT b, a FROM x";
        assert_ne!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
        let q1 = "SELECT b FROM x";
        let q2 = "SELECT b, a FROM x";
        assert_ne!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
    }

    #[test]
    fn it_ignores_insert_col_ordering() {
        let q1 = "INSERT INTO test (a, b) VALUES ($1, $2)";
        let q2 = "INSERT INTO test (b, a) VALUES ($1, $2)";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        // Testing uniqueness
        let q1 = "INSERT INTO test (a, c) VALUES ($1, $2)";
        let q2 = "INSERT INTO test (b, a) VALUES ($1, $2)";
        assert_ne!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
        let q1 = "INSERT INTO test (b) VALUES ($1, $2)";
        let q2 = "INSERT INTO test (b, a) VALUES ($1, $2)";
        assert_ne!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
    }

    #[test]
    fn it_ignores_in_list_size() {
        let q1 = "SELECT * FROM x WHERE y IN ($1, $2, $3)";
        let q2 = "SELECT * FROM x WHERE y IN ($1)";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);

        let q1 = "SELECT * FROM x WHERE y IN ( $1::uuid, $2::uuid, $3::uuid )";
        let q2 = "SELECT * FROM x WHERE y IN ( $1::uuid )";
        assert_eq!(fingerprint(q1).unwrap().hex, fingerprint(q2).unwrap().hex);
    }

    #[test]
    fn it_works() {
        let result = fingerprint("SELECT 1").unwrap();
        assert_eq!(result.hex, "50fde20626009aba");

        let result = fingerprint("SELECT 2").unwrap();
        assert_eq!(result.hex, "50fde20626009aba");

        let result = fingerprint("SELECT $1").unwrap();
        assert_eq!(result.hex, "50fde20626009aba");

        let result = fingerprint("SELECT 1; SELECT a FROM b").unwrap();
        assert_eq!(result.hex, "3efa3b10d558d06d");

        let result = fingerprint("SELECT COUNT(DISTINCT id), * FROM targets WHERE something IS NOT NULL AND elsewhere::interval < now()").unwrap();
        assert_eq!(result.hex, "26b6553101185d22");

        let result = fingerprint("INSERT INTO test (a, b) VALUES ($1, $2)").unwrap();
        assert_eq!(result.hex, "51e63b8083b48bdd");

        let result = fingerprint("INSERT INTO test (b, a) VALUES ($1, $2)").unwrap();
        assert_eq!(result.hex, "51e63b8083b48bdd");

        let result = fingerprint(
        "INSERT INTO test (a, b) VALUES (ARRAY[$1, $2, $3, $4], $5::timestamptz), (ARRAY[$6, $7, $8, $9], $10::timestamptz), ($11, $12::timestamptz)",
    )
    .unwrap();
        assert_eq!(result.hex, "4dfdd5260cac5acf");

        let result = fingerprint("SELECT b AS x, a AS y FROM z").unwrap();
        assert_eq!(result.hex, "1a8bf5d7614de3a5");

        let result = fingerprint("SELECT * FROM x WHERE y = $1").unwrap();
        assert_eq!(result.hex, "4ff39426bd074231");

        let result = fingerprint("SELECT * FROM x WHERE y = ANY ($1)").unwrap();
        assert_eq!(result.hex, "4ff39426bd074231");

        let result = fingerprint("SELECT * FROM x WHERE y IN ($1)").unwrap();
        assert_eq!(result.hex, "4ff39426bd074231");

        let result = fingerprint("SELECT * FROM x WHERE y IN ($1, $2, $3)").unwrap();
        assert_eq!(result.hex, "4ff39426bd074231");

        let result = fingerprint("SELECT * FROM x WHERE y IN ( $1::uuid )").unwrap();
        assert_eq!(result.hex, "4ff39426bd074231");

        let result =
            fingerprint("SELECT * FROM x WHERE y IN ( $1::uuid, $2::uuid, $3::uuid )").unwrap();
        assert_eq!(result.hex, "4ff39426bd074231");

        let result = fingerprint("PREPARE a123 AS SELECT a").unwrap();
        assert_eq!(result.hex, "9b5e6ead8be993e8");

        let result = fingerprint("EXECUTE a123").unwrap();
        assert_eq!(result.hex, "44ef1d2beabd53e8");

        let result = fingerprint("DEALLOCATE a123").unwrap();
        assert_eq!(result.hex, "d8a65a814fbc5f95");

        let result = fingerprint("DEALLOCATE ALL").unwrap();
        assert_eq!(result.hex, "2debfb8745df64a7");

        let result = fingerprint("EXPLAIN ANALYZE SELECT a").unwrap();
        assert_eq!(result.hex, "82845c1b5c6102e5");

        let result =
            fingerprint("WITH a AS (SELECT * FROM x WHERE x.y = $1 AND x.z = 1) SELECT * FROM a")
                .unwrap();
        assert_eq!(result.hex, "6831e38bbb3dd18c");

        let result =
        fingerprint("CREATE TABLE types (a float(2), b float(49), c NUMERIC(2, 3), d character(4), e char(5), f varchar(6), g character varying(7))")
            .unwrap();
        assert_eq!(result.hex, "008d6ba4aa0f4c6e");

        let result =
        fingerprint("CREATE VIEW view_a (a, b) AS WITH RECURSIVE view_a (a, b) AS (SELECT * FROM a(1)) SELECT \"a\", \"b\" FROM \"view_a\"").unwrap();
        assert_eq!(result.hex, "c6ef6b9f498feda4");

        let result = fingerprint("VACUUM FULL my_table").unwrap();
        assert_eq!(result.hex, "fdf2f4127644f4d8");

        let result = fingerprint("SELECT * FROM x AS a, y AS b").unwrap();
        assert_eq!(result.hex, "4e9acae841dae228");

        let result = fingerprint("SELECT * FROM y AS a, x AS b").unwrap();
        assert_eq!(result.hex, "4e9acae841dae228");

        let result = fingerprint("SELECT x AS a, y AS b FROM x").unwrap();
        assert_eq!(result.hex, "65dff5f5e9a643ad");

        let result = fingerprint("SELECT y AS a, x AS b FROM x").unwrap();
        assert_eq!(result.hex, "65dff5f5e9a643ad");

        let result = fingerprint("SELECT x, y FROM z").unwrap();
        assert_eq!(result.hex, "330267237da5535f");

        let result = fingerprint("SELECT y, x FROM z").unwrap();
        assert_eq!(result.hex, "330267237da5535f");

        let result = fingerprint("INSERT INTO films (code, title, did) VALUES ('UA502', 'Bananas', 105), ('T_601', 'Yojimbo', DEFAULT)").unwrap();
        assert_eq!(result.hex, "459fdc70778b841e");

        let result =
            fingerprint("INSERT INTO films (code, title, did) VALUES ($1, $2, $3)").unwrap();
        assert_eq!(result.hex, "459fdc70778b841e");

        let result = fingerprint("SELECT * FROM a").unwrap();
        assert_eq!(result.hex, "fcf44da7b597ef43");

        let result = fingerprint("SELECT * FROM a AS b").unwrap();
        assert_eq!(result.hex, "fcf44da7b597ef43");

        let result =
            fingerprint("UPDATE users SET one_thing = $1, second_thing = $2 WHERE users.id = $1")
                .unwrap();
        assert_eq!(result.hex, "a0ea386c1cfd1e69");

        let result =
            fingerprint("UPDATE users SET something_else = $1 WHERE users.id = $1").unwrap();
        assert_eq!(result.hex, "3172bc3e0d631d55");

        let result = fingerprint("UPDATE users SET something_else = (SELECT a FROM x WHERE uid = users.id LIMIT 1) WHERE users.id = $1").unwrap();
        assert_eq!(result.hex, "f1127a8b91fbecbf");

        let result = fingerprint("SAVEPOINT some_id").unwrap();
        assert_eq!(result.hex, "8ebd566ea1bf947b");

        let result = fingerprint("RELEASE some_id").unwrap();
        assert_eq!(result.hex, "60d618658252d2af");

        let result = fingerprint("PREPARE TRANSACTION 'some_id'").unwrap();
        assert_eq!(result.hex, "d993959a33d627d4");

        let result = fingerprint("START TRANSACTION READ WRITE").unwrap();
        assert_eq!(result.hex, "4ca25828c835d55a");

        let result =
            fingerprint("DECLARE cursor_123 CURSOR FOR SELECT * FROM test WHERE id = 123").unwrap();
        assert_eq!(result.hex, "d2bec62d2a7ec7cb");

        let result = fingerprint("FETCH 1000 FROM cursor_123").unwrap();
        assert_eq!(result.hex, "37f4d2f6a957ae48");

        let result = fingerprint("CLOSE cursor_123").unwrap();
        assert_eq!(result.hex, "2c7963684fc2bad9");

        let result = fingerprint("-- nothing").unwrap();
        assert_eq!(result.hex, "d8d13f8b2da6c9ad");

        let result = fingerprint("CREATE FOREIGN TABLE ft1 () SERVER no_server").unwrap();
        assert_eq!(result.hex, "74481c4af7c76be1");

        let result = fingerprint("UPDATE x SET a = 1, b = 2, c = 3").unwrap();
        assert_eq!(result.hex, "fd5c248c0e642ce4");

        let result = fingerprint("UPDATE x SET z = now()").unwrap();
        assert_eq!(result.hex, "a222eaabaa1e7cb1");

        let result = fingerprint(
            "CREATE TEMPORARY TABLE my_temp_table (test_id integer NOT NULL) ON COMMIT DROP",
        )
        .unwrap();
        assert_eq!(result.hex, "1407ed5c5bb00967");

        let result = fingerprint("CREATE TEMPORARY TABLE my_temp_table AS SELECT 1").unwrap();
        assert_eq!(result.hex, "695ebe73a3abc45c");

        let result = fingerprint("SELECT INTERVAL (0) $2").unwrap();
        assert_eq!(result.hex, "50fde20626009aba");

        let result = fingerprint("SELECT INTERVAL (2) $2").unwrap();
        assert_eq!(result.hex, "50fde20626009aba");

        let result = fingerprint("SELECT * FROM t WHERE t.a IN (1, 2) AND t.b = 3").unwrap();
        assert_eq!(result.hex, "346aea01be9173b6");

        let result = fingerprint("SELECT * FROM t WHERE t.b = 3 AND t.a IN (1, 2)").unwrap();
        assert_eq!(result.hex, "346aea01be9173b6");

        let result = fingerprint("SELECT * FROM t WHERE a && '[1,2]'").unwrap();
        assert_eq!(result.hex, "673f199f13dfe665");

        let result = fingerprint("SELECT * FROM t WHERE a && '[1,2]'::int4range").unwrap();
        assert_eq!(result.hex, "673f199f13dfe665");

        let result = fingerprint("SELECT * FROM t_20210301_x").unwrap();
        assert_eq!(result.hex, "6f8169980cd70a25");

        let result = fingerprint("SELECT * FROM t_20210302_x").unwrap();
        assert_eq!(result.hex, "6f8169980cd70a25");

        let result = fingerprint("SELECT * FROM t_20210302_y").unwrap();
        assert_eq!(result.hex, "d357dac4a24fcf1b");

        let result = fingerprint("SELECT * FROM t_1").unwrap();
        assert_eq!(result.hex, "018bd9230646143e");

        let result = fingerprint("SELECT * FROM t_2").unwrap();
        assert_eq!(result.hex, "3f1444da570c1a66");
    }
}
