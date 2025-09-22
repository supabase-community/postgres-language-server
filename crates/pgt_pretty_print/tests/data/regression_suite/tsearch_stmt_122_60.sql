CREATE INDEX wowidx ON test_tsvector USING gist (a tsvector_ops(siglen=484));
