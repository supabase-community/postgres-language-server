CREATE TABLE errtst(a text, b text NOT NULL, c text, secret1 text, secret2 text) PARTITION BY LIST (a);
