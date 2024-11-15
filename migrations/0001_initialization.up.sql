create table if not exists namespaces (
  id   integer not null,
  name string  not null,

  primary key (id)
);
create unique index if not exists namespaces_name_idx on namespaces(name);

create table if not exists queues (
  id   integer not null,
  ns   integer not null,
  name string  not null,

  primary key (id),
  foreign key (ns) references namespaces(id) on delete cascade
);
create unique index if not exists queues_ns_name_idx on queues(ns, name);

create table if not exists messages (
  id    integer not null,
  queue integer  not null,
  body  blob    not null,
  delivered_at  integer not null default 0,

  primary key (id),
  foreign key (queue) references queues(id) on delete cascade
);
create index if not exists messages_ns_queue_idx on messages(queue);

create table if not exists kv_pairs (
  id      integer not null,
  message integer not null,
  k       string  not null,
  v       string  not null,

  primary key (id),
  foreign key (message) references messages(id) on delete cascade
);
create unique index if not exists kv_message_idx on kv_pairs(message, k);

create table if not exists api_keys (
  id integer not null,
  key_id string not null unique,
  hashed_key text not null,

  primary key (id)
);
create unique index if not exists api_keys_hash_idx on api_keys(key_id, hashed_key);


