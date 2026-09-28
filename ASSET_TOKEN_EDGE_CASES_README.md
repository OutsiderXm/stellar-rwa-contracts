# Asset Token Edge-Case Policy Decisions

This document summarizes the work done to close out four open questions
about `contracts/asset-token/src/lib.rs` edge-case behaviour. In each case
the code's *existing* runtime behaviour was audited, confirmed, then made
**deliberate and documented** (rustdoc on the contract methods + a summary
table in `docs/asset-token.md`), and pinned with a test.

## Issue 1 — Zero-amount transfers

**Decision:** Rejected. `Self::check_amount` reverts with `Error::InvalidAmount`
(#5) for any `amount <= 0`, applied uniformly to `transfer`, `mint`, and
`burn`. A zero-amount call that still emits an event and costs fees while
changing nothing is misleading, so callers must skip the call entirely.

- Documented on `AssetTokenContract::transfer` in `lib.rs`.
- Pinned by the existing `test_zero_amount_rejected` (and
  `test_negative_amount_rejected`) in `contracts/asset-token/src/test.rs`.

## Issue 2 — Self-transfers (`from == to`)

**Decision:** Allowed, but short-circuited into a true no-op — balances are
left untouched (no debit/credit sequence that could double-apply and
inflate the balance), while a `transfer` event is still emitted with
`new_from_bal == new_to_bal == from_bal` so indexers see a consistent event
shape.

- Documented on `AssetTokenContract::transfer` in `lib.rs`.
- Pinned by `test_self_transfer_no_inflation`,
  `test_self_transfer_exceeding_balance_fails`, and
  `test_self_transfer_by_suspended_holder_fails`.

## Issue 3 — Burn by a suspended holder

**Decision:** Rejected. `burn` checks the caller against the compliance
contract exactly like the `from` side of a `transfer`, reverting with
`Error::SenderNotCompliant` (#7) if the caller is currently suspended or
removed. Burn still mutates balance/total-supply state, so it stays behind
the same compliance gate as every other balance-changing call — a
suspended holder cannot use burn to self-service an exit.

- Documented on `AssetTokenContract::burn` in `lib.rs`.
- Newly pinned by `test_burn_blocked_when_holder_suspended` (added in this
  change, using `compliance.suspend` specifically, alongside the existing
  `test_burn_blocked_when_holder_not_compliant`, which uses
  `compliance.remove`).

## Issue 4 — Mint gates the recipient

**Decision:** Yes, `mint` (and `mint_batch`, per recipient) already checks
the recipient against the compliance contract and reverts with
`Error::RecipientNotCompliant` (#8) if it fails. This was confirmed
deliberate: minting is the only way new supply enters circulation, so
leaving it ungated would let tokens reach an address no `transfer` could
ever reach.

- Documented on `AssetTokenContract::mint` in `lib.rs`.
- Documented in `docs/asset-token.md` under the new "Edge-case policy
  decisions" table.
- Pinned by the existing `test_mint_to_noncompliant_fails` and
  `test_mint_batch_reverts_entirely_on_noncompliant_recipient`.

## Files touched

- `contracts/asset-token/src/lib.rs` — rustdoc explaining each decision on
  `transfer`, `mint`, and `burn`.
- `contracts/asset-token/src/test.rs` — one new test pinning burn-while-
  suspended behaviour.
- `docs/asset-token.md` — new "Edge-case policy decisions" table.
