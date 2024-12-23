drop index api_keys_key_id_idx;
drop index api_keys_user_name_idx;
drop table api_keys;

drop index user_permissions_user_namespace_idx;
drop table user_permissions;

drop index sessions_kv_idx;
drop table session_state;

drop index sessions_key_idx;
drop table sessions;

drop index kv_message_idx;
drop table kv_pairs;

drop index messages_queue_idx;
drop table messages;

drop index queue_configs_queue_idx;
drop table queue_configurations;

drop index queues_ns_name_idx;
drop table queues;

drop index namespaces_name_idx;
drop table namespaces;

drop index users_email_idx;
drop table users;
