-- An exclusion constraint on a new table locks nothing that other sessions use.
-- expect_no_diagnostics
create table bookings (room int, during tstzrange, exclude using gist (room with =, during with &&));
