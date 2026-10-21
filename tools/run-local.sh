#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
: "${GHOSTLIGHT_MODEL:?Set GHOSTLIGHT_MODEL to the model ID served by your local model server}"
export GHOSTLIGHT_LOCAL_ENDPOINT="${GHOSTLIGHT_LOCAL_ENDPOINT:-127.0.0.1:8080}"
export GHOSTLIGHT_CONTROLLER_PROJECTOR_MODEL="${GHOSTLIGHT_CONTROLLER_PROJECTOR_MODEL:-local/$GHOSTLIGHT_MODEL}"
export GHOSTLIGHT_CONTROLLER_PERSONA_MODEL="${GHOSTLIGHT_CONTROLLER_PERSONA_MODEL:-local/$GHOSTLIGHT_MODEL}"
export GHOSTLIGHT_CONTROLLER_INTERPRETER_MODEL="${GHOSTLIGHT_CONTROLLER_INTERPRETER_MODEL:-local/$GHOSTLIGHT_MODEL}"
export GHOSTLIGHT_CONTROLLER_OPERATIONAL_MODEL="${GHOSTLIGHT_CONTROLLER_OPERATIONAL_MODEL:-local/$GHOSTLIGHT_MODEL}"
export GHOSTLIGHT_CONTROLLER_ELABORATOR_MODEL="${GHOSTLIGHT_CONTROLLER_ELABORATOR_MODEL:-local/$GHOSTLIGHT_MODEL}"
export GHOSTLIGHT_PLAY_MODEL="${GHOSTLIGHT_PLAY_MODEL:-local/$GHOSTLIGHT_MODEL}"
state_root="${GHOSTLIGHT_STATE_ROOT:-$HOME/.local/share/mcp-ghostlight/world}"
if [ -x target/debug/ghostlight-dungeon ]; then
    exec target/debug/ghostlight-dungeon --local --state-root "$state_root"
fi
exec cargo run --locked -p ghostlight-dungeon --bin ghostlight-dungeon -- --local --state-root "$state_root"
