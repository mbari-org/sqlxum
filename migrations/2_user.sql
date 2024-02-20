-- See src/models/mod.rs

create table usr
(
    user_id       uuid primary key       default uuid_generate_v1mc(),
    email         text unique not null,
    name          text                   not null,
    created_at    timestamptz            not null default now(),
    updated_at    timestamptz
);

SELECT trigger_updated_at('usr');
