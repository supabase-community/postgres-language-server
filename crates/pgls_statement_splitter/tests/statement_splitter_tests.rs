use std::fs::{self};

const DATA_DIR_PATH: &str = "tests/data/";

#[test]
fn test_statement_splitter() {
    let mut paths: Vec<_> = fs::read_dir(DATA_DIR_PATH)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for f in paths.iter() {
        let path = f.path();
        let test_name = path.file_stem().unwrap().to_str().unwrap();
        let expected_count = test_name
            .split("__")
            .last()
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let contents = fs::read_to_string(&path).unwrap();

        let split = pgls_statement_splitter::split(&contents);

        assert_eq!(
            split.ranges.len(),
            expected_count,
            "Mismatch in statement count for file {}: Expected {} statements, got {}",
            test_name,
            expected_count,
            split.ranges.len()
        );
    }
}
