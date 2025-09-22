CREATE PUBLICATION testpub6 FOR TABLE rf_bug WHERE (status = 'open') WITH (publish = 'insert');
