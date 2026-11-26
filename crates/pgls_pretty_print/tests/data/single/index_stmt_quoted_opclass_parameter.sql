-- An operator class parameter name that needs quotes keeps them.
CREATE INDEX ON t USING gist (a gist_ops ("Sig len" = 8));
