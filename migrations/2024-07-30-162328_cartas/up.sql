drop table if exists cartas;
drop table if exists users;

create table cartas (
    id serial primary key not null, -- sequential
    parent integer, -- tree-style replies. null designates root node
    user_id integer, -- null designates anonymous
    title bpchar(24), -- null for non-root nodes
    content character varying(2048) not null,
    modification_code char(6) not null, -- 6-digit "pin"
    -- creation timestamp not null,
    -- modification timestamp
    -- fixme! `timestamp`s to [`chrono::NaiveDateTime`] doesn't meet trait bounds
    creation integer not null, -- unix timestamp
    modification integer, -- unix timestamp
    random_accessible bool not null
);

create table users (
    id serial primary key not null, -- sequential
    certificate_hash bytea not null, -- sha256 hash
    -- fixme! `timestamp`s to [`chrono::NaiveDateTime`] doesn't meet trait bounds
    creation integer not null -- unix timestamp
)
