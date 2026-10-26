-- NOT VALID goes before the comment and the comma.
-- expect_lint/constraintMissingNotValid
-- expect_lint/constraintMissingNotValid
alter table orders
    add constraint orders_amount_check check (amount > 0) -- positive amounts only
    , add constraint orders_customer_fk foreign key (customer_id) references customers (id) on delete cascade deferrable initially deferred;
