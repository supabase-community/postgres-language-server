CREATE INDEX wowidx1 ON test_tsvector USING gist (a tsvector_ops(siglen=0));
