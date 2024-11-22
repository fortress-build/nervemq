create table if not exists users (
  id integer not null,
  email text not null unique,
  hashed_pass text not null,
  role text not null check (role in ('admin', 'user')),

  primary key(id)
);
create unique index if not exists users_email_idx on users(email);

create table if not exists namespaces (
  id   integer not null,
  name string  not null,
  created_by integer not null,

  primary key (id),
  foreign key (created_by) references users(id)
);
create unique index if not exists namespaces_name_idx on namespaces(name);

create table if not exists queues (
  id   integer not null,
  ns   integer not null,
  name string  not null,
  created_by integer,

  primary key (id),
  foreign key (ns) references namespaces(id) on delete cascade,
  foreign key (created_by) references users(id) on delete set null
);
create unique index if not exists queues_ns_name_idx on queues(ns, name);

create table if not exists queue_configurations (
  id integer not null,
  queue integer not null,
  max_retries integer not null,
  dead_letter_queue integer,

  primary key (id),
  foreign key (queue) references queues(id) on delete cascade,
  foreign key (dead_letter_queue) references queues(id) on delete cascade
);
create unique index if not exists queue_configs_queue_idx on queue_configurations(queue);

create table if not exists messages (
  id    integer not null,
  queue integer not null,
  body  blob    not null,
  delivered_at  integer,
  sent_by       integer,
  tries integer not null default 0,

  primary key (id),
  foreign key (queue) references queues(id) on delete cascade,
  foreign key (sent_by) references users(id) on delete set null
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

create table if not exists sessions (
  id integer not null,
  session_key text not null,
  ttl integer not null,

  primary key (id)
);
create unique index if not exists sessions_key_idx on sessions(session_key);

create table if not exists session_state (
  session integer not null,
  k text not null,
  v text not null,

  primary key (session, k),
  foreign key (session) references sessions(id) on delete cascade
);
create unique index if not exists sessions_kv_idx on session_state(session, k);

-- TODO: Proper RBAC
--
-- create table if not exists permissions (
--   id integer not null,
--   scope string not null,
--   name string not null,
-- );
-- create unique index on permissions(scope, name);

create table if not exists user_permissions (
  id integer not null,
  user integer not null,
  namespace integer not null,
  can_delete_ns boolean default false,

  primary key (id),
  foreign key (user) references users(id) on delete cascade,
  foreign key (namespace) references namespaces(id) on delete cascade
);
create unique index if not exists user_permissions_user_namespace_idx on user_permissions(user, namespace);

create table if not exists api_keys (
  id integer not null,
  user integer not null,
  name string not null,
  key_id text not null,
  hashed_key text not null,

  primary key (id),
  foreign key (user) references users(id) on delete cascade
);
create unique index if not exists api_keys_user_name_idx on api_keys(user, name);
create unique index if not exists api_keys_key_id_idx on api_keys(key_id);
