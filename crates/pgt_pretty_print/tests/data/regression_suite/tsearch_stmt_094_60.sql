CREATE INDEX wowidx2 ON test_tsvector USING gist (a tsvector_ops(siglen=1));
