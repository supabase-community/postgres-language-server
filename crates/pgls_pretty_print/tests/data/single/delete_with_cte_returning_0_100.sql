WITH stale AS (
  SELECT id
  FROM sessions
  WHERE last_seen < now() - INTERVAL '30 days'
)
DELETE FROM sessions
USING stale
WHERE sessions.id = stale.id
RETURNING sessions.id;
