-- pgls-format: layout=expanded, indentStyle=tabs, indentSize=4, keywordCase=upper
CREATE TABLE cleaning.allocation_keys AS
WITH pre_merged AS (
    SELECT allocation_keys.id
    FROM refining.allocation_keys
)
SELECT id
FROM pre_merged;
