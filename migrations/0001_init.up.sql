create table namespaces (
  id   integer not null,
  name string  not null,

  primary key (id)
);
create unique index namespaces_name_idx on namespaces(name);

create table queues (
  id   integer not null,
  ns   integer not null,
  name string  not null,

  primary key (id),
  foreign key (ns) references namespaces(id)
);
create unique index queues_ns_name_idx on queues(ns, name);

create table messages (
  id    integer not null,
  queue integer not null,
  body  blob    not null,
  delivered_at  integer not null default 0,

  primary key (id),
  foreign key (queue) references queues(id)
);
create unique index messages_queue_idx on messages(queue);

create table kv_pairs (
  id      integer not null,
  message integer not null,
  k       string  not null,
  v       string  not null,

  primary key (id),
  foreign key (message) references messages(id)
);
create unique index kv_message_idx on kv_pairs(message);
