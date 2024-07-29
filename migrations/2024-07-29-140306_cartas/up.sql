drop table if exists cartas;

create table cartas (
    id serial primary key not null, -- sequential
    parent integer, -- tree-style replies. null designates root node
    user_id integer, -- maybe null
    content character varying(2048) not null,
    modification_code char(6) -- 6-digit "pin"
);
