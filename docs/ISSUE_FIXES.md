# Issue fixes: decimals bounds, dividend storage docs, registry dedup, mint_batch cost

This note documents four small, independent changes made together on one
branch (one commit per issue).

## Issue 1 — extreme `decimals` behavior is now documented by a test

`AssetTokenContract::initialize` (`contracts/asset-token/src/lib.rs`) accepts
any `u32` for `decimals` with no upper-bound validation. There was no test
exercising a large `decimals` value combined with a large `total_supply` /
`valuation`.

Added `test_extreme_decimals_accepted_with_large_supply_and_valuation` in
`contracts/asset-token/src/test.rs`, which initializes a token with
`decimals = 255` and a `total_supply` / `valuation` near `i128::MAX / 2`, and
asserts that:
- `initialize` does not panic or clamp `decimals`,
- `get_metadata`, `balance`, and `total_supply` stay internally consistent,
- a subsequent `mint` still works normally.

This documents *current* behavior (accepted, unbounded) so that if an upper
bound is added later, the test's expectations change visibly rather than the
gap being silently closed or silently regressed. `decimals` is opaque
metadata to asset-token itself; the real risk (e.g. `dividend`'s
`total_amount * basis` style math, or a UI scaling by `10^decimals`) lives in
downstream consumers, not in this contract's own arithmetic.

## Issue 2 — `docs/dividend.md` storage table updated to match current `DataKey`

The `Storage / TTL` table in `docs/dividend.md` still reflected the
pre-snapshot storage layout and was missing `DataKey::Snapshot`,
`DataKey::Supply`, and `DataKey::AssetIds`, all added by the snapshot-based
distribution rewrite (issue #163) and the per-asset index (issue #166) in
`contracts/dividend/src/lib.rs`.

Regenerated the table by hand against the current enum and code paths:
- Added rows for `AssetIds(Address)`, `Supply(u64)`, and `Snapshot(u64)`,
  each noting when they're written, TTL-extended, and (for `Supply` /
  `Snapshot`) removed on distribution completion.
- Filled in previously-"unknown" rows (`Ids`, `Claimed`) with their actual
  behavior: `Ids` is an unused legacy variant kept for storage-key stability;
  `Claimed(u64, Address)` is set once per claim with no explicit
  `extend_ttl` call.

## Issue 3 — registry regression test for duplicate `token_contract` registration

`contracts/registry/src/test.rs` had no test covering registering the same
`token_contract` address twice under different issuers/names.
`register_asset` (`contracts/registry/src/lib.rs`) has no dedup check
(tracked separately as a design gap).

Added
`test_register_same_token_contract_twice_creates_two_entries_and_double_counts_tvl`,
which registers the same `token_contract` under two different issuers/names
and asserts:
- both registrations succeed with distinct asset ids,
- `asset_count()` is 2,
- both entries reference the same `token_contract` but have different
  issuers/names,
- `total_value_locked()` sums both valuations — i.e. the same underlying
  token contract is double-counted in TVL today.

This locks in current behavior so that if/when dedup is added, this test
will need (and visibly prompt) an update, rather than silently continuing to
pass against changed semantics.

## Issue 4 — `mint_batch` doc comment now states its cross-contract cost model

`AssetTokenContract::mint_batch` (`contracts/asset-token/src/lib.rs`) calls
`Self::compliant` — a cross-contract call into the compliance contract —
once per recipient inside its loop, so its resource cost and cross-contract
call count both scale linearly with the recipient list size. This wasn't
mentioned in the function's doc comment.

Added a "Cost model" paragraph to the doc comment stating this explicitly,
so callers batching large recipient lists know up front to budget resource
limits accordingly and to consider splitting very large batches across
multiple calls.
