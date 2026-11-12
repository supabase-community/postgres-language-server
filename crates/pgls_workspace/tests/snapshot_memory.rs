//! Measures how much memory the schema snapshot of a database takes, which the workspace keeps per
//! database connection. Needs the test database (`DATABASE_URL`). Run with
//! `cargo test -p pgls_workspace --release --test snapshot_memory -- --ignored --nocapture`.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use pgls_catalog::Snapshot;
use sqlx::{Executor, PgPool};

/// Counts the bytes currently allocated.
struct Counting;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// The bytes a loaded snapshot of the database at `url` keeps allocated.
async fn snapshot_bytes(url: &str) -> usize {
    let pool = PgPool::connect(url).await.unwrap();
    // Warm up the pool and sqlx' statement cache, which aren't part of the snapshot.
    drop(Snapshot::load(&pool).await.unwrap());
    let before = ALLOCATED.load(Ordering::Relaxed);
    let snapshot = Snapshot::load(&pool).await.unwrap();
    let retained = ALLOCATED.load(Ordering::Relaxed).saturating_sub(before);
    println!(
        "{url}: {} tables, {} columns, {} functions, {} types: {:.1} KiB",
        snapshot.tables.len(),
        snapshot.columns.len(),
        snapshot.functions.len(),
        snapshot.types.len(),
        retained as f64 / 1024.0
    );
    drop(snapshot);
    pool.close().await;
    retained
}

#[tokio::test]
#[ignore]
async fn measure_snapshot_memory() {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@127.0.0.1:5432/postgres".to_string());
    snapshot_bytes(&url).await;

    // A larger schema: 500 tables with 10 columns each, 200 functions, 50 views and 20 enums.
    let admin = PgPool::connect(&url).await.unwrap();
    let database = "pgls_snapshot_memory";
    admin
        .execute(format!("drop database if exists {database}").as_str())
        .await
        .unwrap();
    admin
        .execute(format!("create database {database}").as_str())
        .await
        .unwrap();
    let large_url = format!("{}/{database}", url.rsplit_once('/').unwrap().0);
    let large = PgPool::connect(&large_url).await.unwrap();
    let mut ddl = String::new();
    for table in 0..500 {
        let columns: Vec<_> = (0..10).map(|column| format!("c{column} text")).collect();
        ddl.push_str(&format!(
            "create table t{table} (id bigint primary key, {});\n",
            columns.join(", ")
        ));
    }
    for function in 0..200 {
        ddl.push_str(&format!(
            "create function f{function}(a int, b text) returns int language sql as 'select a';\n"
        ));
    }
    for view in 0..50 {
        ddl.push_str(&format!("create view v{view} as select * from t{view};\n"));
    }
    for kind in 0..20 {
        ddl.push_str(&format!("create type e{kind} as enum ('a', 'b', 'c');\n"));
    }
    large.execute(ddl.as_str()).await.unwrap();
    large.close().await;

    snapshot_bytes(&large_url).await;

    admin
        .execute(format!("drop database {database}").as_str())
        .await
        .unwrap();
}
