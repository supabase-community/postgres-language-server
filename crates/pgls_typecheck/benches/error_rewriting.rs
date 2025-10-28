use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pgls_typecheck::IdentifierReplacement;
use pgls_typecheck::diagnostics::rewrite_error_message;

fn benchmark_error_rewriting(c: &mut Criterion) {
    let replacement = IdentifierReplacement {
        original_name: "user_id".to_string(),
        original_range: 0..7,
        default_value: "'00000000-0000-0000-0000-000000000000'".to_string(),
        type_name: "uuid".to_string(),
    };

    // test case 1: matching the first pattern (most common case)
    c.bench_function("rewrite_invalid_input_syntax", |b| {
        b.iter(|| {
            rewrite_error_message(
                black_box(r#"invalid input syntax for type integer: "00000000-0000-0000-0000-000000000000""#),
                black_box(&replacement),
            )
        })
    });

    // test case 2: matching the operator pattern
    c.bench_function("rewrite_operator_does_not_exist", |b| {
        b.iter(|| {
            rewrite_error_message(
                black_box("operator does not exist: integer + text"),
                black_box(&replacement),
            )
        })
    });

    // test case 3: no pattern matches (fallback)
    c.bench_function("rewrite_fallback", |b| {
        b.iter(|| {
            rewrite_error_message(
                black_box("some other error message that doesn't match any pattern"),
                black_box(&replacement),
            )
        })
    });

    // test case 4: longer error message with first pattern
    c.bench_function("rewrite_long_message", |b| {
        b.iter(|| {
            rewrite_error_message(
                black_box(r#"invalid input syntax for type timestamp: "00000000-0000-0000-0000-000000000000" at character 45"#),
                black_box(&replacement),
            )
        })
    });
}

criterion_group!(benches, benchmark_error_rewriting);
criterion_main!(benches);
