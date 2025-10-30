## Manual installation

Directly installing the standalone CLI can be a great alternative if you are not already using a package manager.

### Supported platforms

You have to pick the correct binary for your platform. The following table should help you do so.

| CPU Architecture | Operating System | Binary name |
|------------------|------------------|-------------|
| x86_64 | Linux | `postgres-language-server_x86_64-unknown-linux-gnu` |
| aarch64 | Linux | `postgres-language-server_aarch64-unknown-linux-gnu` |
| x86_64 | macOS | `postgres-language-server_x86_64-apple-darwin` |
| aarch64 (M1/M2) | macOS | `postgres-language-server_aarch64-apple-darwin` |
| x86_64 | Windows | `postgres-language-server_x86_64-pc-windows-msvc.exe` |
| aarch64 | Windows | `postgres-language-server_aarch64-pc-windows-msvc.exe` |

> **Note**: Use the Linux variant for Windows Subsystem for Linux (WSL).


### Homebrew

Postgres Language Server is available as a [Homebrew formula](https://formulae.brew.sh/formula/postgres-language-server) for macOS and Linux users.

```sh
brew install postgres-language-server
```

### Using a published binary

To install postgres-language-server, grab the executable for your platform from the latest CLI release on GitHub and give it execution permission.

```bash
# macOS arm (M1 or newer)
curl -L https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgres-language-server_aarch64-apple-darwin -o postgres-language-server
chmod +x postgres-language-server

# macOS x86_64
curl -L https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgres-language-server_x86_64-apple-darwin -o postgres-language-server
chmod +x postgres-language-server

# Linux (x86_64)
curl -L https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgres-language-server_x86_64-unknown-linux-gnu -o postgres-language-server
chmod +x postgres-language-server

# Linux (aarch64)
curl -L https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgres-language-server_aarch64-unknown-linux-gnu -o postgres-language-server
chmod +x postgres-language-server

# Windows (x86_64, PowerShell)
Invoke-WebRequest -Uri "https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgres-language-server_x86_64-pc-windows-msvc.exe" -OutFile "postgres-language-server.exe"

# Windows (aarch64, PowerShell)
Invoke-WebRequest -Uri "https://github.com/supabase-community/postgres-language-server/releases/latest/download/postgres-language-server_aarch64-pc-windows-msvc.exe" -OutFile "postgres-language-server.exe"
```

Now you can use the Postgres Language Server by simply running `./postgres-language-server`.
