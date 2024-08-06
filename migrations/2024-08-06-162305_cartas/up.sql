drop table if exists cartas;

create table cartas (
    id serial primary key not null, -- sequential
    uuid char(36) not null, -- 32-len uuid + 4 hyphens
    parent integer, -- tree-style replies. null designates root node
    user_id integer, -- null designates anonymous
    title bpchar(36), -- null for unknown
    sender bpchar(24), -- null for unknown
    content character varying(2048) not null,
    modification_code char(6) not null, -- 6-digit "pin"
    -- creation timestamp not null,
    -- modification timestamp
    -- fixme! `timestamp`s to [`chrono::NaiveDateTime`] doesn't meet trait bounds
    creation integer not null, -- unix timestamp
    modification integer, -- unix timestamp
    lang char(2) not null, -- language code
    random_accessible bool not null,
    reports integer not null,
    ip character varying(45) not null -- ipv6 is 45 char max!!
);

insert into cartas values(
    0, -- id
    '00000000-0000-0000-0000-000000000000', -- uuid
    null, -- parent
    null, -- user id
    'welcome', -- title
    'sheepy', -- sender
    'welcome to the Abyss. i hope you enjoy your stay.', -- content
    '370569', -- modification code
    1722892738, -- creation time
    null, -- modification time
    'en', -- lang
    false, -- random accessible
    0, -- reports
    '127.0.0.1'
)
