# Local single-user development

The fork supports an explicit local API host without Odin, Heimdall, Idunn,
Discord, or CultMesh. It reuses Dungeon's Eve commands, play table, mailbox,
and kernel. The managed startup path remains the default.

## Start

With Rust installed, from the repository root:

```sh
GHOSTLIGHT_MODEL=your-served-model tools/run-local.sh
```

The model must be available on an OpenAI-compatible server at
`127.0.0.1:8080` with `/v1/chat/completions` and tool-calling support. Override
`GHOSTLIGHT_LOCAL_ENDPOINT` with a loopback socket address if necessary. The
script selects the `local/` route for all six model lanes. This repository
does not install or launch a model server.

The API listens at `127.0.0.1:8831`. Override `GHOSTLIGHT_LOCAL_BIND` to another
loopback socket address. Non-loopback addresses and managed Idunn environment
bindings are refused. Runtime data defaults to
`$HOME/.local/share/mcp-ghostlight/world`; override `GHOSTLIGHT_STATE_ROOT` to
use a separate world. Do not point this mode at a production store.

The equivalent binary invocation is:

```sh
target/debug/ghostlight-dungeon --local --state-root "$HOME/.local/share/mcp-ghostlight/world"
```

All model-lane environment settings must already be set for a direct binary
invocation; see the launcher. The existing Claude sidecar transport remains
available when configured explicitly instead of the local model route.

## API access

At first startup, the host creates `secrets/local-api.token` under the state
root, containing 32 random bytes with mode 0600 on Unix. Requests use its
**hex representation** in `Authorization: Bearer <hex>`. The token grants
one stable local owner identity. No request may choose another identity.
There are no local login or logout commands. Stop the server and remove the
token file to rotate access; restart creates a new token without changing
the world's owner.

For a local probe (keep this token out of logs and chat):

```sh
export GHOSTLIGHT_STATE_ROOT="${GHOSTLIGHT_STATE_ROOT:-$HOME/.local/share/mcp-ghostlight/world}"
python3 - <<'PY'
import json, os, pathlib, urllib.request
root = pathlib.Path(os.environ['GHOSTLIGHT_STATE_ROOT'])
token = (root / 'secrets/local-api.token').read_bytes().hex()
request = urllib.request.Request(
    'http://127.0.0.1:8831/api/eve/surfaces/ghostlight.play',
    headers={'Authorization': 'Bearer ' + token})
with urllib.request.urlopen(request) as response:
    print(json.dumps(json.load(response), indent=2))
PY
```

The local host serves the API only, not the browser application. The existing
browser's Heimdall sign-in adapter does not manage local bearer tokens. An MCP
adapter should keep the token outside model-visible tool inputs and outputs.

## MCP adapter contract

- `GET /api/eve/provider` publishes provider metadata.
- `GET /api/eve/surfaces/ghostlight.play` returns the authorized player view
  and command descriptors. Without a valid token it returns the anonymous
  surface, as the managed host does without a session.
- `POST /api/eve/commands` accepts the existing Eve command envelope. Use its
  advertised schema, transport, current source version and a UUID idempotency
  key. HTTP 200 alone does not mean acceptance: inspect `receipt.state`,
  including `denied` and `unknown`.
- `world.begin` creates, approves, and (without a configured seed vault)
  activates a world. With a vault it stops in approved Draft so `world.seed`
  can run before `world.activate`.
- `world.play` admits a turn and executes asynchronously. Re-read the
  player surface for narration, faults and open questions. Answers must use
  the currently offered answer token. Reuse the same idempotency key and body
  for a transport retry; never turn an unknown result into a fresh action.
- `GET /api/eve/events` signals that the view may have changed. Event counters
  are wake signals, not command source versions. Re-read the surface.

This adds no general world snapshot, historical event export or Saganode
import route. The consumer patch endpoint retains its existing confinement.

## Studio-box builds without a host Rust installation

Use an official Rust container and mount only this checkout and a Cargo cache:

```sh
mkdir -p "$HOME/.cache/ghostlight-cargo"
docker run --rm --user "$(id -u):$(id -g)" \
  -e CARGO_HOME=/cargo \
  -v "$HOME/.cache/ghostlight-cargo:/cargo" \
  -v "$PWD:/workspace" -w /workspace \
  rust:1.90-bookworm@sha256:3914072ca0c3b8aad871db9169a651ccfce30cf58303e5d6f2db16d1d8a7e58f \
  cargo build --locked -p ghostlight-dungeon --bin ghostlight-dungeon
```

Run the built Linux binary on the host to reach its loopback model server.
This is local fork validation, not an Idunn production release.
