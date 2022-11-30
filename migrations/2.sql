create table if not exists key_limit (
    id serial primary key,
    "key" varchar(64) not null unique,
    "limit" int not null
);

insert into key_limit ("key", "limit") values
    ('one', 1),
    ('five', 5),
    ('ten', 10),
    ('tri', 33);