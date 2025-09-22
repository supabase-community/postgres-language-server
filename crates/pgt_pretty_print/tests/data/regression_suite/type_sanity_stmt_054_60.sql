SELECT indexrelid::regclass
FROM pg_index
WHERE (is_catalog_text_unique_index_oid(indexrelid) <>
       (indisunique AND
        indexrelid < 16384 AND
        EXISTS (SELECT 1 FROM pg_attribute
                WHERE attrelid = indexrelid AND atttypid = 'text'::regtype)));
