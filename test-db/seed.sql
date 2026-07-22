-- Help line

create table public.contact (
    id serial primary key not null,
    created_at timestamp with time zone not null default now(),
    username text
);

