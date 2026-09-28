# Deployments

This file records the deployed contract ids for each network so that anyone
can audit a deployment against the repository.

## Testnet

| Contract    | Id |
| ----------- | -- |
| compliance  | _TBD_ |
| registry    | _TBD_ |
| dividend    | _TBD_ |
| asset-token | _TBD_ |

Deployments are produced by `scripts/deploy.sh`; copy the printed ids into the
table above.

## Verifying a deployment

To confirm that a deployed contract matches a given commit of this repository,
use `scripts/verify-deployment.sh`. It builds the wasm at the requested commit
and compares its hash against the on-chain wasm hash of the deployed contract.

```sh
# Verify a contract against the current HEAD
./scripts/verify-deployment.sh <contract-id>

# Verify a contract against a specific commit
./scripts/verify-deployment.sh <contract-id> <commit>

# Verify against a non-default network
NETWORK=testnet ./scripts/verify-deployment.sh <contract-id> <commit>
```

The script prints `MATCH` when the locally built wasm matches the deployed
contract and `MISMATCH` otherwise, exiting non-zero on a mismatch so it can be
used in CI or audit scripts.

Requirements: `stellar` CLI (>= 22), `git`, and `sha256sum` (or `shasum`).
