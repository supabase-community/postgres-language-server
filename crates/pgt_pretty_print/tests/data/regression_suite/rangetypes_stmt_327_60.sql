create domain restrictedrange as int4range check (upper(value) < 10);
