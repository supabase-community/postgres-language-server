create table public.accounts (
  -- expect_lint/identifierTooLong
  "Customer Subscription Billing Cycle Adjustments History Archive Id" bigint,
  -- expect_lint/identifierTooLong
  customer_subscription_billing_cycle_adjustments_history_archive_note text,
  -- Unquoted names are downcased before they are truncated.
  -- expect_lint/identifierTooLong
  Customer_Subscription_Billing_Cycle_Adjustments_History_Archive_Ref text
);
