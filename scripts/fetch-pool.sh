#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "Verwendung: $0 <rpc-url> <pool-adresse> [radius]" >&2
  exit 1
fi

RPC_URL="$1"
POOL="$2"
RADIUS="${3:-3}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INSPECT="$ROOT/target/release/clmm-inspect"
OUT="$ROOT/accounts/$POOL"

if [ ! -x "$INSPECT" ]; then
  echo "clmm-inspect fehlt, bitte zuerst 'cargo build --release' ausführen" >&2
  exit 1
fi

mkdir -p "$OUT"

rpc() {
  curl -sS --fail -X POST -H "Content-Type: application/json" -d "$1" "$RPC_URL"
}

rpc "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"$POOL\",{\"encoding\":\"base64\",\"commitment\":\"confirmed\"}]}" > "$OUT/pool.json"

mapfile -t ADDRESSES < <("$INSPECT" addresses "$POOL" "$OUT/pool.json" "$RADIUS")
printf '%s\n' "${ADDRESSES[@]}" > "$OUT/related-addresses.txt"

LIST=$(printf '"%s",' "${ADDRESSES[@]}")
LIST="${LIST%,}"
rpc "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getMultipleAccounts\",\"params\":[[${LIST}],{\"encoding\":\"base64\",\"commitment\":\"confirmed\"}]}" > "$OUT/related.json"

echo "Reihenfolge in related.json: [0] AmmConfig, [1] Bitmap-Extension, [2..] Tick-Arrays von -$RADIUS bis +$RADIUS um den aktuellen Tick"
echo
"$INSPECT" decode "$OUT/pool.json" "$OUT/related.json"
