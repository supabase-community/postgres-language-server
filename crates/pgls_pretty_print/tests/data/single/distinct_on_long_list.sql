SELECT DISTINCT ON (addresses.hash) addresses.hash AS origin_hash, addresses.entity_type, addresses.entity_fk
FROM normalization.addresses
ORDER BY addresses.hash, addresses.entity_type;
