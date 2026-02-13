SELECT pg_notify('notify_async1','sample message1');

SELECT pg_notify('notify_async1','');

SELECT pg_notify('notify_async1',NULL);

SELECT pg_notify('','sample message1');

SELECT pg_notify(NULL,'sample message1');

SELECT pg_notify('notify_async_channel_name_too_long______________________________','sample_message1');

NOTIFY notify_async2;

LISTEN notify_async2;

UNLISTEN notify_async2;

UNLISTEN *;

SELECT pg_notification_queue_usage();
