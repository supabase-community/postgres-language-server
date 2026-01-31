SELECT
  100 * 3 + (vs.i - 1) * 3 AS offset_a,
  (metrics.total - metrics.used) / (metrics.total + metrics.used) AS utilization,
  cost + (tax_rate * subtotal) - (discount_rate * subtotal) AS net_total
FROM
  balances AS vs,
  ledger AS metrics,
  invoices
WHERE
  (amount + fee) * (1 - discount) > (limit_value - buffer)
  AND (temperature - ambient) * factor > threshold;
