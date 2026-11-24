//! What the scale tests share: a generated file of any number of statements, and an allocator
//! that counts what each thread allocates.
//!
//! The scale tests assert operation counts instead of timings. A count is the same on every run
//! and every machine, so a budget can be tight: it is today's count plus about 5 %, and a change
//! that makes something cheaper lowers the budget to its new count.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

/// A generated file in the shape of a `pg_dump`: per table a `CREATE TABLE`, a comment, a primary
/// key, an index, a foreign key to the previous table, an `INSERT`, a view, a PL/pgSQL function,
/// a grant and a query, ten statements in all.
pub struct Dump {
    pub sql: String,
    /// The offset of each statement.
    starts: Vec<usize>,
}

/// The statements per table of a [`Dump`].
pub const DUMP_UNIT_STATEMENTS: usize = 10;

impl Dump {
    /// A dump of `statements` statements.
    pub fn new(statements: usize) -> Self {
        Self::generate(statements, None)
    }

    /// A dump of `statements` statements, with the statements of each table in a transaction:
    /// a `BEGIN` before and a `COMMIT` after, which aren't counted in `statements`.
    pub fn with_transactions(statements: usize) -> Self {
        Self::generate(statements, Some(("begin;", "commit;")))
    }

    fn generate(statements: usize, transaction: Option<(&str, &str)>) -> Self {
        let mut dump = Self {
            sql: String::new(),
            starts: Vec::with_capacity(statements),
        };
        let mut table = 0;
        let mut added = 0;
        while added < statements {
            let unit = Self::unit(table);
            let take = unit.len().min(statements - added);
            if let Some((begin, _)) = transaction {
                dump.push(begin);
            }
            for statement in unit.iter().take(take) {
                dump.push(statement);
            }
            if let Some((_, commit)) = transaction {
                dump.push(commit);
            }
            added += take;
            table += 1;
        }
        dump
    }

    fn push(&mut self, statement: &str) {
        if !self.sql.is_empty() {
            self.sql.push_str("\n\n");
        }
        self.starts.push(self.sql.len());
        self.sql.push_str(statement);
    }

    /// The statements about table `i`, as `gen.py` of the performance review writes them.
    fn unit(i: usize) -> [String; DUMP_UNIT_STATEMENTS] {
        let p = i.saturating_sub(1);
        [
            format!(
                "create table public.t{i} (\n    id bigint not null,\n    parent_id bigint,\n    name text not null,\n    email character varying(255),\n    created_at timestamp with time zone default now() not null,\n    amount numeric(12,2) default 0\n);"
            ),
            format!("comment on table public.t{i} is 'table number {i}';"),
            format!("alter table only public.t{i}\n    add constraint t{i}_pkey primary key (id);"),
            format!("create index t{i}_name_idx on public.t{i} using btree (name, created_at);"),
            format!(
                "alter table only public.t{i}\n    add constraint t{i}_parent_fk foreign key (parent_id) references public.t{p}(id) on delete cascade;"
            ),
            format!(
                "insert into public.t{i} (id, parent_id, name, email, amount) values (1, null, 'a', 'a@example.com', 1.5), (2, 1, 'b', null, 2);"
            ),
            format!(
                "create view public.v{i} as\n select t.id, t.name, p.name as parent_name, count(*) over (partition by t.parent_id) as siblings\n   from (public.t{i} t\n     left join public.t{p} p on ((p.id = t.parent_id)))\n  where (t.amount > (0)::numeric);"
            ),
            format!(
                "create function public.f{i}(p_id bigint) returns text\n    language plpgsql stable\n    as $$\ndeclare\n  v_name text;\nbegin\n  select name into v_name from public.t{i} where id = p_id;\n  if v_name is null then\n    raise exception 'missing %', p_id;\n  end if;\n  return upper(v_name);\nend;\n$$;"
            ),
            format!("grant select on table public.t{i} to authenticated;"),
            format!(
                "select t.id, t.name, f{i}(t.id) from public.t{i} t join public.v{i} v on v.id = t.id where t.created_at > now() - interval '1 day' order by t.id limit 10;"
            ),
        ]
    }

    /// The text of each statement, in file order.
    pub fn statement_texts(&self) -> impl Iterator<Item = &str> {
        let ends = self
            .starts
            .iter()
            .skip(1)
            .map(|next| next - "\n\n".len())
            .chain(std::iter::once(self.sql.len()));
        self.starts
            .iter()
            .zip(ends)
            .filter_map(|(&start, end)| self.sql.get(start..end))
    }

    /// The number of statements, `BEGIN` and `COMMIT` included.
    pub fn statements(&self) -> usize {
        self.starts.len()
    }

    /// The tables, one per ten statements.
    pub fn tables(&self) -> usize {
        self.sql.matches("create table ").count()
    }

    /// The PL/pgSQL functions, one per ten statements.
    pub fn functions(&self) -> usize {
        self.sql.matches("create function ").count()
    }

    /// Where the scale tests type a space: into whitespace, so the statement stays valid, as in
    /// checkpoint 0. In the first statement (after `create`), and before the `from` of the query
    /// of the middle table and of the last one. Empty if the dump has no query.
    pub fn edit_offsets(&self) -> Vec<usize> {
        let first = self.starts.first().map(|start| start + "create".len());
        let queries = [0.5, 1.0].map(|fraction| {
            let query = self.query(fraction)?;
            Some(query + self.sql.get(query..)?.find(" from ")?)
        });
        match (first, queries) {
            (Some(first), [Some(middle), Some(last)]) => vec![first, middle, last],
            _ => Vec::new(),
        }
    }

    /// The offset of the query (`select t.id, …`) of the table about `fraction` of the way into
    /// the file: 0.0 is the first table's, 1.0 the last one's. `None` if the dump has no query.
    pub fn query(&self, fraction: f64) -> Option<usize> {
        let queries: Vec<usize> = self
            .sql
            .match_indices("select t.id, t.name, f")
            .map(|(offset, _)| offset)
            .collect();
        let last = queries.len().checked_sub(1)?;
        let index = (last as f64 * fraction.clamp(0.0, 1.0)).round() as usize;
        queries.get(index).copied()
    }
}

thread_local! {
    static ALLOCATED: Cell<Allocations> = const {
        Cell::new(Allocations { count: 0, bytes: 0, freed: 0 })
    };
}

/// An allocator that counts what each thread allocates and frees, for [`Allocations::during`].
/// A test binary uses it with
///
/// ```ignore
/// #[global_allocator]
/// static ALLOCATOR: pgls_test_utils::scale::CountingAllocator =
///     pgls_test_utils::scale::CountingAllocator;
/// ```
///
/// It counts per thread, so the tests of a binary don't count each other's allocations. Work
/// that a request hands to another thread isn't counted.
pub struct CountingAllocator;

impl CountingAllocator {
    fn add(count: usize, bytes: usize, freed: usize) {
        // `try_with`: the allocator also runs while the thread's locals are destroyed.
        let _ = ALLOCATED.try_with(|allocated| {
            let now = allocated.get();
            allocated.set(Allocations {
                count: now.count + count,
                bytes: now.bytes + bytes,
                freed: now.freed + freed,
            });
        });
    }
}

// SAFETY: every call is forwarded to `System` unchanged; counting doesn't allocate.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Self::add(1, layout.size(), 0);
        // SAFETY: the caller's contract is `System.alloc`'s.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        Self::add(1, layout.size(), 0);
        // SAFETY: the caller's contract is `System.alloc_zeroed`'s.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        Self::add(0, 0, layout.size());
        // SAFETY: the caller's contract is `System.dealloc`'s.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // As in the performance review's spikes: a reallocation counts as an allocation of the
        // new size.
        Self::add(1, new_size, layout.size());
        // SAFETY: the caller's contract is `System.realloc`'s.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// What a thread allocated, as counted by [`CountingAllocator`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Allocations {
    /// Allocations and reallocations.
    pub count: usize,
    /// Bytes allocated.
    pub bytes: usize,
    /// Bytes freed.
    pub freed: usize,
}

impl Allocations {
    /// Runs `f` and returns what this thread allocated meanwhile. Without [`CountingAllocator`]
    /// as the global allocator, it counts nothing.
    pub fn during<T>(f: impl FnOnce() -> T) -> (T, Allocations) {
        let before = ALLOCATED.with(Cell::get);
        let value = f();
        let after = ALLOCATED.with(Cell::get);
        (
            value,
            Allocations {
                count: after.count - before.count,
                bytes: after.bytes - before.bytes,
                freed: after.freed - before.freed,
            },
        )
    }

    /// The bytes still held of what was allocated: allocated minus freed, which is negative if
    /// more was freed than allocated.
    pub fn live(&self) -> isize {
        self.bytes as isize - self.freed as isize
    }
}
