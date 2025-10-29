mod deparse;
mod error;
mod fingerprint;
mod iter_mut;
mod iter_ref;
mod node_enum;
mod node_mut;
mod node_ref;
mod node_structs;
mod normalize;
mod parse;
mod plpgsql;
mod scan;
mod split;

pub use deparse::*;
pub use error::*;
pub use fingerprint::*;
pub use iter_mut::*;
pub use iter_ref::*;
pub use node_enum::*;
pub use node_mut::*;
pub use node_ref::*;
pub use normalize::*;
pub use parse::*;
pub use plpgsql::*;
pub use scan::*;
pub use split::*;

pub use protobuf::Node;

// Include the generated bindings with 2024 edition compatibility
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(improper_ctypes)]
#[allow(unsafe_op_in_unsafe_fn)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Include the generated protobuf code
#[allow(clippy::all)]
pub mod protobuf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/protobuf.rs"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_does_not_error_when_run_in_parallel() {
        use easy_parallel::Parallel;

        let mut queries = vec![];
        for _ in 0..100 {
            queries.push(
            r#"
            SELECT * FROM "t0"
            JOIN "t1" ON (1) JOIN "t2" ON (1) JOIN "t3" ON (1) JOIN "t4" ON (1) JOIN "t5" ON (1)
            JOIN "t6" ON (1) JOIN "t7" ON (1) JOIN "t8" ON (1) JOIN "t9" ON (1) JOIN "t10" ON (1)
            JOIN "t11" ON (1) JOIN "t12" ON (1) JOIN "t13" ON (1) JOIN "t14" ON (1) JOIN "t15" ON (1)
            JOIN "t16" ON (1) JOIN "t17" ON (1) JOIN "t18" ON (1) JOIN "t19" ON (1) JOIN "t20" ON (1)
            JOIN "t21" ON (1) JOIN "t22" ON (1) JOIN "t23" ON (1) JOIN "t24" ON (1) JOIN "t25" ON (1)
            JOIN "t26" ON (1) JOIN "t27" ON (1) JOIN "t28" ON (1) JOIN "t29" ON (1)
        "#,
        );
            queries.push(
            "
            SELECT memory_total_bytes, memory_free_bytes, memory_pagecache_bytes, memory_buffers_bytes, memory_applications_bytes,
                (memory_swap_total_bytes - memory_swap_free_bytes) AS swap, date_part($0, s.collected_at) AS collected_at
            FROM snapshots s JOIN system_snapshots ON (snapshot_id = s.id)
            WHERE s.database_id = $0 AND s.collected_at BETWEEN $0 AND $0
            ORDER BY collected_at
        ",
        );
        }

        Parallel::new()
            .each(queries, |query| {
                for _ in 0..100 {
                    let _result = parse(query).unwrap();
                    fingerprint(query).unwrap();
                    normalize(query).unwrap();
                }
            })
            .run();
    }
}
