-- format specs of Rust
select {:>8}, {0:+#010.3e}, {name:.*}, {:x?}, {:<1$} from t;
-- found by the fuzzer: `$$` is no format spec, so `{:$$}` starts no directive. Read as one, it ran
-- over the `$$` that ends the string
sElect($${:$$})select('');$$)
