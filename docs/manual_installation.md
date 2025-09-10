## Manual installation

Directly installing the standalone CLI can be a great alternative if you are not already using a package manager.

### Supported platforms

You have to pick the correct binary for your platform. The following table should help you do so.

| CPU Architecture | Operating System | Binary name |
|------------------|------------------|-------------|
| x86_64 | Linux | `postgrestools-x86_64-unknown-linux-gnu` |
| aarch64 | Linux | `postgrestools-aarch64-unknown-linux-gnu` |
| x86_64 | macOS | `postgrestools-x86_64-apple-darwin` |
| aarch64 (M1/M2) | macOS | `postgrestools-aarch64-apple-darwin` |
| x86_64 | Windows | `postgrestools-x86_64-pc-windows-msvc.exe` |
| aarch64 | Windows | `postgrestools-aarch64-pc-windows-msvc.exe` |

> **Note**: Use the Linux variant for Windows Subsystem for Linux (WSL).


### Homebrew

We were not able to publish to Homebrew yet due to naming conflicts. We are actively working to resolve this.

### Using a published binary

To install postgrestools, grab the executable for your platform from the latest CLI release on GitHub and give it execution permission.

```bash
# macOS arm (M1 or newer)
curl -L https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgrestools-aarch64-apple-darwin -o postgrestools
chmod +x postgrestools

# macOS x86_64
curl -L https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgrestools-x86_64-apple-darwin -o postgrestools
chmod +x postgrestools

# Linux (x86_64)
curl -L https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgrestools-x86_64-unknown-linux-gnu -o postgrestools
chmod +x postgrestools

# Linux (aarch64)
curl -L https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgrestools-aarch64-unknown-linux-gnu -o postgrestools
chmod +x postgrestools

# Windows (x86_64, PowerShell)
Invoke-WebRequest -Uri "https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgrestools-x86_64-pc-windows-msvc.exe" -OutFile "postgrestools.exe"

# Windows (aarch64, PowerShell)
Invoke-WebRequest -Uri "https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgrestools-aarch64-pc-windows-msvc.exe" -OutFile "postgrestools.exe"
```

Now you can use the Postgres Language Server by simply running `./postgrestools`.
