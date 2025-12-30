WITH src AS (
  SELECT 1 AS id, 'alpha' AS name
)
INSERT INTO audit.log (id, name)
SELECT id, name
FROM src
RETURNING id, name;
