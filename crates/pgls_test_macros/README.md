# Tests macros

Macros to help auto-generate tests based on files.

## Usage

Pass a glob pattern that'll identify your files and a test-function that'll run for each file. The glob pattern has to start at the root of your crate.

You can add a `.expected.` file next to your test file. Its path will be passed to your test function so you can make outcome-based assertions. (Alternatively, write snapshot tests.)

Given the following file structure:

```txt
crate/
|-- src/
|-- tests/
    |-- queries/
        |-- test.sql
        |-- test.expected.sql
    |-- querytest.rs
```

You can generate tests like so:

```rust
  // crate/tests/querytest.rs

  tests_macros::gen_tests!{
    "tests/queries/*.sql",
    crate::run_test  // use `crate::` if the linter complains.
  }

  fn run_test(
    test_path: &str,  // absolute path on the machine
    expected_path: &str, // absolute path of .expected file
    test_dir: &str // absolute path of the test file's parent
  ) {
      // your logic
  }
```

Given a `crate/tests/queries/some_test_abc.sql` file, this will generate the following:

```rust
#[test]
pub fn some_test_abc()
{
    let test_file = "<crate>/tests/queries/some_test_abc.sql";
    let test_expected_file = "<crate>/tests/queries/some_test_abc.expected.sql";
    let parent = "<crate>/tests/queries";
    run_test(test_file, test_expected_file, parent);
}
```

This will be replicated for each file matched by the glob pattern.

## Pitfalls

- If you use a Rust-keyword as a file name, this'll result in invalid syntax for the generated tests.
- You might get linting errors if your test files aren't snake case.
- All files of the glob-pattern must (currently) be `.sql` files.
- The `.expected.sql` file-name will always be passed, even if the file doesn't exist.
- The macro will wrap your tests in a `mod tests { .. }` module. If you need multiple generations, wrap them in modules like so: `mod some_test { tests_macros::gen_tests! { .. } }`.

## How to run

Simply run your `cargo test` commands as usual.
