drop table if exists users;

create table users (
    id serial primary key not null, -- sequential
    certificate_hash bytea not null, -- sha256 hash
    lang char(2) not null, -- language code
    -- fixme! `timestamp`s to [`chrono::NaiveDateTime`] doesn't meet trait bounds
    creation integer not null -- unix timestamp
)
