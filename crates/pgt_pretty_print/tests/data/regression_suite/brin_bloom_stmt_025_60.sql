CREATE INDEX brin_summarize_bloom_idx ON brin_summarize_bloom USING brin (value) WITH (pages_per_range=2);
