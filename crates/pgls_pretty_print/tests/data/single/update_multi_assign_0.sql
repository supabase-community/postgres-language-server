UPDATE accounts
SET (balance, updated_at) = (balance + delta, now())
FROM adjustments
WHERE accounts.id = adjustments.account_id;
