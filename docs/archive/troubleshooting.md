## Troubleshooting

This guide describes how to resolve common issues with Postgres Language Tools.

### Incorrect and / or misplaced diagnostics

We are employing pragmatic solutions to split a SQL file into statements, and they might be incorrect in certain cases. If you see diagnostics like `Unexpected token` in the middle of a valid statement, make sure to either end all statements with a semicolon, or put two double newlines between them. If there are still issues, its most likely a bug in the change handler that is gone after reopening the file. But please file an issue with sample code so we can fix the root cause.


