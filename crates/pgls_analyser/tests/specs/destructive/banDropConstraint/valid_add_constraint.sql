-- expect_no_diagnostics
alter table users add constraint users_email_key unique (email);
alter domain positive_int add constraint positive_int_check check (value > 0);
