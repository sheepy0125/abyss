-- for some reason, using `timestamp`s & serializing to [`chrono::NaiveDateTime`]s
-- isn't working. i don't *really* wanna debug it, so i'll switch to unix timestamps.

drop table if exists cartas;

create table cartas (
    id serial primary key not null, -- sequential
    parent integer, -- tree-style replies. null designates root node
    user_id integer, -- null designates anonymous
    content character varying(2048) not null,
    modification_code char(6) not null -- 6-digit "pin"
    -- creation timestamp not null,
    -- modification timestamp not null
);
