WITH pending AS (
  SELECT id
  FROM invoices
  WHERE status = 'pending'
)
UPDATE invoices AS inv
SET status = 'processed'
FROM pending
WHERE inv.id = pending.id
RETURNING inv.id, inv.status;
