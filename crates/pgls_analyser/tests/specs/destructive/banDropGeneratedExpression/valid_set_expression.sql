-- expect_no_diagnostics
alter table line_items alter column total set expression as (price * quantity);
