
-- update dependencies for langchain

UPDATE app.packages SET control_requires = '{vector}' WHERE handle = 'langchain';

INSERT INTO app.package_upgrades(package_id, from_version_struct, to_version_struct, sql)
VALUES (
(SELECT id FROM app.packages WHERE handle = 'langchain' AND partial_name = 'embedding_search'),
(1,1,0),
(1,1,1),
$langchain$
SELECT pg_extension_config_dump('documents', '');
SELECT pg_extension_config_dump('documents_id_seq', '');
$langchain$
);

INSERT INTO app.package_upgrades(package_id, from_version_struct, to_version_struct, sql)
VALUES (
(SELECT id FROM app.packages WHERE handle = 'langchain' AND partial_name = 'hybrid_search'),
(1,1,0),
(1,1,1),
$langchain$
SELECT pg_extension_config_dump('documents', '');
SELECT pg_extension_config_dump('documents_id_seq', '');
$langchain$
);

INSERT INTO app.package_upgrades(package_id, from_version_struct, to_version_struct, sql)
VALUES (
(SELECT id FROM app.packages WHERE partial_name = 'pg_headerkit'),
(1,0,0),
(1,0,1),
$hdr$
SELECT pg_extension_config_dump('hdr.allow_list', '');
SELECT pg_extension_config_dump('hdr.deny_list', '');
$hdr$
);
