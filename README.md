# djv.sh

Personal homepage built with [Leptos](https://github.com/leptos-rs/leptos) and [Axum](https://github.com/tokio-rs/axum).

## Development

Enter the dev shell and start the server:

```bash
nix develop
just dev
```

For remote development, tunnel ports 3000 (app) and 3001 (hot-reload):

```bash
ssh -L 3000:localhost:3000 -L 3001:localhost:3001 user@host
```

### Commands

```bash
just check  # format, lint, and test (Rust + Nix)
just fmt    # format code
just lint   # run clippy, statix, deadnix
just test   # run tests
just build  # release build
```

## Deployment (NixOS)

### 1. Add flake input

```nix
{
  inputs.djv.url = "github:djvcom/djv/stable";
}
```

The `stable` tag points to the latest commit that passed the full CI build pipeline, including the Nix package build. This ensures your NixOS config only updates to known-good versions.

### 2. Import module and enable service

```nix
{ inputs, ... }:
{
  imports = [ inputs.djv.nixosModules.default ];

  services.djv = {
    enable = true;
    environment = "production";
  };
}
```

### 3. Configure nginx

```nix
services.nginx.virtualHosts."djv.sh" = {
  forceSSL = true;
  enableACME = true;
  locations."/" = {
    proxyPass = "http://unix:/run/djv/djv.sock";
    proxyWebsocketUpgrade = true;
  };
};
```

### Module options

| Option | Default | Description |
|--------|---------|-------------|
| `services.djv.enable` | `false` | Enable the service |
| `services.djv.environment` | `"production"` | Deployment environment (`deployment.environment.name`) |
| `services.djv.socketPath` | `"/run/djv/djv.sock"` | Unix socket path |
| `services.djv.opentelemetryEndpoint` | `"http://127.0.0.1:4318"` | OTel collector endpoint |

## Observability

Traces are exported via OTLP to the configured endpoint. Resource attributes include:

- `service.name` / `service.version` - from Cargo.toml
- `deployment.environment.name` - from module config
- `vcs.ref.head.revision` / `vcs.ref.head.name` - git commit and branch at build time

## Licence

MIT
