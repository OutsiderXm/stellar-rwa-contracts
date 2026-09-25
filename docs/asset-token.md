# Asset Token — SEP-41 Conformance Audit

This document tracks conformance of `contracts/asset-token/src/lib.rs` against
the [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
fungible token interface, as required for the web app's allowance-reading
distribution flow.

## Method-by-method audit

| SEP-41 method | Present | Signature match | Notes |
|---|---|---|---|
| `allowance(from, spender) -> i128` | Yes | Yes | Returns `0` once `expiration_ledger` has passed instead of a stale positive value. |
| `approve(from, spender, amount, expiration_ledger)` | Yes | Yes | Additionally rejected while `paused` (divergence, see below). |
| `balance(id) -> i128` | Yes | Yes | Matches spec. |
| `transfer(from, to, amount)` | Yes | Yes | Additionally gated on the compliance contract for both parties (divergence, see below). |
| `transfer_from(spender, from, to, amount)` | Yes | Yes | Same compliance gating as `transfer`. |
| `burn(from, amount)` | Yes | Yes | Matches spec. |
| `burn_from(spender, from, amount)` | No | — | Not implemented. The web app's distribution flow only reads `allowance`/`transfer_from`; this is a known gap, not audited further here. |
| `decimals() -> u32` | Partial | — | Exposed via `get_metadata().decimals` rather than a top-level `decimals()` fn. Divergence: metadata bundles decimals with other asset fields the web app already reads in one call. |
| `name() -> String` | Partial | — | Exposed via `get_metadata().name`, same reasoning as `decimals`. |
| `symbol() -> String` | Partial | — | Exposed via `get_metadata().symbol`, same reasoning as `decimals`. |

## Documented divergences

1. **Compliance gating on `transfer`/`transfer_from`.** SEP-41 does not
   define a compliance hook. This contract requires both the sender and
   recipient to pass `is_allowed` on the configured compliance contract
   before any balance moves. This is intentional: the asset represents a
   real-world asset subject to KYC/AML restrictions, and the spec's
   `transfer`/`transfer_from` signatures are otherwise preserved unchanged.
2. **`approve` rejected while paused.** The spec does not require this, but
   allowing new approvals during a pause would let `transfer_from` calls
   queue up and fire the instant the token is unpaused, defeating the
   purpose of pausing. `allowance` reads still work while paused.
3. **`name`/`symbol`/`decimals` are metadata fields, not top-level
   functions.** The web app already fetches `get_metadata()` once per asset;
   splitting these into separate calls would only add round trips.
4. **`burn_from` is not implemented.** No caller in this codebase currently
   needs delegated burning. Adding it is a follow-up if that changes.

## Test coverage

`contracts/asset-token/src/test.rs` covers:
- `approve` followed by `transfer_from` moving the approved amount.
- `transfer_from` rejected once the approved amount is exceeded.
- `transfer_from` rejected once `expiration_ledger` has passed (expiry).
- `allowance` reading back `0` for an expired approval.
