#!/usr/bin/env bash
#
# verify_deployment.sh - Verify that a deployed Soroban contract's on-chain
# wasm hash matches the wasm built from a given commit of this repository.
#
# Usage:
#   scripts/verify_deployment.sh <commit> <contract-id> [network]
#
# Arguments:
#   commit       Git commit (or ref) to build and compare against.
#   contract-id  Deployed contract id (C...).
#   network      Stellar network to query (default: testnet).
#
# Environment:
#   STELLAR_RPC_URL  Optional RPC URL override passed to the Stellar CLI.
#
# Exit codes:
#   0  The deployed wasm hash matches the wasm built at <commit>.
#   1  Mismatch, or a required tool/argument is missing.
#
set -euo pipefail

usage() {
  echo "Usage: $0 <commit> <contract-id> [network]" >&2
  echo "Example: $0 v1.2.3 CABC...XYZ testnet" >&2
}

if [ "$#" -lt 2 ]; then
  usage
  exit 1
fi

COMMIT="$1"
CONTRACT_ID="$2"
NETWORK="${3:-testnet}"

for tool in git cargo stellar; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: required tool '$tool' not found in PATH" >&2
    exit 1
  fi
done

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if ! git rev-parse --verify --quiet "${COMMIT}^{commit}" >/dev/null; then
  echo "error: '$COMMIT' is not a valid commit in this repository" >&2
  exit 1
fi

WORKTREE="$(mktemp -d)"
cleanup() {
  git worktree remove --force "$WORKTREE" >/dev/null 2>&1 || rm -rf "$WORKTREE"
}
trap cleanup EXIT

echo "==> Checking out ${COMMIT} into a temporary worktree"
git worktree add --detach "$WORKTREE" "$COMMIT" >/dev/null

# Build the wasm at the requested commit. `stellar contract build` writes the
# optimized wasm to target/wasm32-unknown-unknown/release/*.wasm.
echo "==> Building wasm at ${COMMIT}"
(
  cd "$WORKTREE"
  stellar contract build >/dev/null
)

WASM_FILE="$(find "$WORKTREE/target/wasm32-unknown-unknown/release" -maxdepth 1 -name '*.wasm' -type f | head -n 1)"
if [ -z "$WASM_FILE" ]; then
  echo "error: no wasm artifact produced by 'stellar contract build'" >&2
  exit 1
fi

LOCAL_HASH="$(sha256sum "$WASM_FILE" | awk '{print $1}')"

RPC_ARGS=()
if [ -n "${STELLAR_RPC_URL:-}" ]; then
  RPC_ARGS=(--rpc-url "$STELLAR_RPC_URL")
fi

# Fetch the wasm currently deployed for the contract and hash it locally so we
# compare like-for-like against the freshly built artifact.
echo "==> Fetching deployed wasm for ${CONTRACT_ID} on ${NETWORK}"
DEPLOYED_WASM="$(mktemp)"
trap 'cleanup; rm -f "$DEPLOYED_WASM"' EXIT
stellar contract fetch \
  --id "$CONTRACT_ID" \
  --network "$NETWORK" \
  "${RPC_ARGS[@]}" \
  --out-file "$DEPLOYED_WASM" >/dev/null

DEPLOYED_HASH="$(sha256sum "$DEPLOYED_WASM" | awk '{print $1}')"

echo ""
echo "commit ${COMMIT} wasm sha256:   ${LOCAL_HASH}"
echo "deployed ${CONTRACT_ID} sha256: ${DEPLOYED_HASH}"
echo ""

if [ "$LOCAL_HASH" = "$DEPLOYED_HASH" ]; then
  echo "MATCH: deployed wasm matches the wasm built at ${COMMIT}"
  exit 0
fi

echo "MISMATCH: deployed wasm does NOT match the wasm built at ${COMMIT}" >&2
exit 1
