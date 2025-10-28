# Environment Variables

[//]: # (BEGIN ENV_VARS)


### `PGLS_LOG_PATH`

 The directory where the Daemon logs will be saved.

### `PGLS_LOG_LEVEL`

 Allows to change the log level. Default is debug. This will only affect "pgls*" crates. All others are logged with info level.

### `PGLS_LOG_PREFIX_NAME`

 A prefix that's added to the name of the log. Default: `server.log.`

### `PGLS_CONFIG_PATH`

 A path to the configuration file

### `PGT_LOG_PATH`

 The directory where the Daemon logs will be saved. Deprecated, use PGLS_LOG_PATH instead.

### `PGT_LOG_LEVEL`

 Allows to change the log level. Default is debug. This will only affect "pgls*" crates. All others are logged with info level. Deprecated, use PGLS_LOG_LEVEL instead.

### `PGT_LOG_PREFIX_NAME`

 A prefix that's added to the name of the log. Default: `server.log`. Deprecated, use PGLS_LOG_PREFIX_NAME instead.

### `PGT_CONFIG_PATH`

 A path to the configuration file. Deprecated, use PGLS_CONFIG_PATH instead.


[//]: # (END ENV_VARS)
