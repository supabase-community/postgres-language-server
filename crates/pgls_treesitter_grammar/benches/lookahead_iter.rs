use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tree_sitter::Language;

/**
 * On MacBook Pro M2 with 16GB Ram, this takes about 1.8 microseconds
 */
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "loading grammar and getting lookahead for many parse states",
        |b| {
            b.iter(|| {
                let lang: Language = pgls_treesitter_grammar::LANGUAGE.into();

                let mut lh_iterator = lang
                    .lookahead_iterator(black_box(32))
                    .expect("Invalid Parse State");

                // contains about 106 nodes for ParseState 32
                let it: Vec<&'static str> = lh_iterator.iter_names().collect();

                black_box(it);
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
