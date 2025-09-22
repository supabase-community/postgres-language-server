CREATE INDEX wowidx1 ON test_tsvector USING gist (a tsvector_ops(foo=1));
