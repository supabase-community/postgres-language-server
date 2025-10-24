-- Valid: Using a lookup table instead of enum
-- expect_no_diagnostics
CREATE TABLE document_type (
    type_name TEXT PRIMARY KEY
);

INSERT INTO document_type VALUES ('invoice'), ('receipt'), ('other');

CREATE TABLE document (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    type TEXT REFERENCES document_type(type_name)
);
