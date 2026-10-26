-- expect_lint/constraintMissingNotValid
-- expect_lint/constraintMissingNotValid
alter table orders
    add constraint orders_amount_check check (amount > 0),
    add constraint orders_user_fk foreign key (user_id) references users (id) on delete cascade deferrable;
