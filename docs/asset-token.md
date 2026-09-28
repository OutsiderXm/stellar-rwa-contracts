# Asset Token Contract

A compliant token representing a tokenized real-world asset. Every `transfer`
checks the compliance contract for **both** sender and recipient, and every
`mint` checks the recipient — so only KYC-approved addresses can hold the asset.

- Testnet: `CBMCWLSQSWUTLUJFCNBHNBSXMUM3XU7NAQ5TSNERW4HA4ZZBYHLG4ECZ`

## The compliance check (core feature)

`transfer` and `mint` call into the compliance contract via a lightweight
generated client:

```rust
#[contractclient(name = "ComplianceClient")]
pub trait ComplianceInterface {
    fn is_allowed(env: Env, address: Address) -> bool;
}
// inside transfer:
if !ComplianceClient::new(&env, &meta.compliance_contract).is_allowed(&from) {
    // -> SenderNotCompliant (#7)
}
if !ComplianceClient::new(&env, &meta.compliance_contract).is_allowed(&to) {
    // -> RecipientNotCompliant (#8)
}
```

This decouples the two contracts at build time — the token only knows the
compliance *interface*, and the concrete compliance contract address is stored
in metadata and can be swapped with `set_compliance`.

## `AssetMetadata`

| Field                 | Type      | Meaning                              |
|-----------------------|-----------|--------------------------------------|
| `name` / `symbol`     | `String`  | Display name and ticker              |
| `asset_type`          | `String`  | `real_estate`, `invoice`, `commodity`|
| `total_supply`        | `i128`    | Current supply (base units)          |
| `decimals`            | `u32`     | Token decimals                       |
| `admin`               | `Address` | Controls mint/pause/valuation        |
| `compliance_contract` | `Address` | Gate consulted on transfer/mint      |
| `asset_description`   | `String`  | Free-text description                |
| `valuation`           | `i128`    | Asset value in **USD cents**         |
| `paused`              | `bool`    | When true, transfers/mints revert    |

## Functions

- `initialize(admin, name, symbol, asset_type, total_supply, decimals, compliance_contract, asset_description, valuation)` —
  stores metadata and mints `total_supply` to `admin`. The admin must already be
  compliance-approved. Admin auth. Once only.
- `transfer(from, to, amount)` — `from` auth; not paused; both parties compliant;
  `from` has balance; moves tokens.
- `mint(admin, to, amount)` — admin auth; not paused; `to` compliant; increases
  supply.
- `burn(from, amount)` — `from` auth; reduces caller balance and supply.
- `balance(id) -> i128`
- `total_supply() -> i128`
- `pause(admin)` / `unpause(admin)` — admin auth.
- `get_metadata() -> AssetMetadata`
- `update_valuation(admin, new_valuation)` — admin auth.
- `set_compliance(admin, compliance)` — admin auth; repoints the gate.
- `propose_admin(admin, new_admin)` — admin auth; records a pending successor.
  The role does not move yet.
- `accept_admin(new_admin)` — pending successor's auth; completes the handover.
- `cancel_admin_proposal(admin)` — admin auth; clears the pending successor.
  See [issue #4](fixes/issue-4.md) for the rationale.

## Swapping compliance mid-life

`set_compliance` repoints the gate at a different contract. The token stores only
the compliance *address*; it does not snapshot or migrate any approval state.
Because approvals live in the compliance contract, not in the token, the set of
addresses that pass `is_allowed` is entirely determined by whichever contract is
currently referenced. Repointing the gate therefore **silently changes who can
transact**:

- Addresses approved under the old contract may not be approved under the new
  one. Their existing balances remain, but their `transfer`/`mint` calls will
  start reverting with `SenderNotCompliant` (#7) or `RecipientNotCompliant` (#8).
- Addresses that were *not* approved under the old contract may become approved
  under the new one, gaining the ability to receive or move the asset.
- The change takes effect immediately for the next `transfer`/`mint`; there is no
  grace period and no per-address migration. `burn` is unaffected (it does not
  consult compliance).
- The swap is not reversible in terms of state: repointing back to the old
  contract restores the old approval set only if that contract's state is
  unchanged.

### Recommended migration procedure

1. **Stage the new compliance contract** and populate it with the intended
   approval set (KYC/allow-list) before touching the token.
2. **Diff the approval sets** off-chain: compute the addresses approved under the
   old contract and under the new one, and identify addresses that would lose
   approval.
3. **Notify affected holders** and complete any required re-approval (KYC) so
   they are approved under the new contract *before* the swap.
4. **Pause the token** (`pause`) to halt transfers/mints while the gate is being
   changed, avoiding a window where some holders are unexpectedly blocked.
5. **Call `set_compliance(admin, new)`** (admin auth). This emits `setcomp`.
6. **Verify** by checking `get_metadata().compliance_contract` and probing a few
   known addresses with the new contract's `is_allowed`.
7. **Unpause** (`unpause`) once the new gate is confirmed correct.

Keep the old compliance contract deployed and unchanged until the migration is
confirmed, so the swap can be rolled back by repointing to it if needed.

## Errors

| Code | Name                   | Cause                                |
|------|------------------------|--------------------------------------|
| 1    | AlreadyInitialized     | double init                          |
| 2    | NotInitialized         | used before init                     |
| 3    | Unauthorized           | non-admin admin-only call            |
| 4    | InsufficientBalance    | transfer/burn over balance           |
| 5    | InvalidAmount          | amount <= 0 (or negative supply/val) |
| 6    | Paused                 | transfer/mint while paused           |
| 7    | SenderNotCompliant     | sender fails `is_allowed`            |
| 8    | RecipientNotCompliant  | recipient fails `is_allowed`         |
| 9    | Overflow               | supply overflow on mint              |

## Events

| Topic       | Data                    | When         |
|-------------|-------------------------|--------------|
| `mint`      | (to) → amount           | mint / init  |
| `transfer`  | (from, to) → amount     | transfer     |
| `burn`      | (from) → amount         | burn         |
| `pause`     | admin                   | pause        |
| `unpause`   | admin                   | unpause      |
| `valuation` | new valuation           | valuation up |
| `setcomp`   | compliance address      | gate changed |
| `set_admin` | (old_admin) → new_admin | admin handed over |

## Storage / TTL

Listing of the contract `DataKey` variants and their storage behaviour.

| Key | Payload | Storage | TTL / Notes |
|-----|---------|---------|-------------|
| `Metadata` | - | instance | - |
| `Balance` | Address | unknown | - |

## Security considerations

- Compliance is enforced **inside** `transfer`/`mint`; it cannot be bypassed by
  calling the token directly.
- Amounts must be strictly positive; zero/negative amounts revert.
- `mint` overflow is checked; supply cannot wrap.
- Only the admin can pause, mint, change valuation, or repoint compliance.
- Repointing compliance with `set_compliance` changes the effective approval set
  immediately; see "Swapping compliance mid-life" above for the operational
  consequences and the recommended migration procedure.

## Failure mode: unreachable compliance contract (issue #305)

`transfer`, `mint`, `mint_batch`, `burn`, and `set_compliance` all call into
`compliance_contract` via the generated `ComplianceClient`. This is a real
cross-contract call, not a local check, so it inherits the failure modes of
any Soroban invocation:

- **No contract deployed at that address** — the host cannot resolve the
  call and **traps**, aborting the entire transaction. No balance, supply, or
  metadata change is applied.
- **The callee traps internally** (e.g. it panics on unexpected input) —
  same result: the trap propagates up, the whole transaction rolls back.
- **The callee returns a value but not `bool`** — this cannot happen without
  bypassing the SDK's type-checked client; if it somehow did, decoding would
  itself trap.

This is the deliberate, and only sane, behavior: the token contract has no
way to distinguish "compliance said no" from "compliance is broken," so it
treats an unreachable or malfunctioning gate as a hard failure rather than
either failing open (allowing the transfer) or silently no-opping. Operators
must ensure `compliance_contract` always points at a live, correctly
implemented contract; `set_compliance` mitigates this somewhat by calling the
new gate before switching to it, but does not protect against the gate later
being removed or bricked.

Proven by test: `test_gate_traps_when_compliance_address_has_no_contract` and
`test_transfer_traps_when_compliance_contract_is_unreachable` in
`contracts/asset-token/src/test.rs`.

## Edge-case policy decisions

These behaviours were previously implicit/accidental; they are now
deliberate and pinned by tests in `contracts/asset-token/src/test.rs`.

| Case | Decision | Rationale | Test |
|------|----------|-----------|------|
| Zero-amount `transfer`/`mint`/`burn` | **Rejected** with `InvalidAmount` (#5) | A no-op call that still emits an event and costs fees is misleading; callers must skip the call instead. | `test_zero_amount_rejected` |
| Self-transfer (`from == to`) | **Allowed**, short-circuited to a true no-op (balances untouched, event still emitted) | Rejecting it forces callers to special-case an address match themselves; a no-op is safe and simpler, and avoids a double-apply bug in naive debit/credit code. | `test_self_transfer_no_inflation`, `test_self_transfer_exceeding_balance_fails`, `test_self_transfer_by_suspended_holder_fails` |
| Burn by a suspended/non-compliant holder | **Rejected** with `SenderNotCompliant` (#7) | `burn` still mutates balance and total supply, so it is gated exactly like the `from` side of a `transfer`; suspension cannot be bypassed via self-burn. | `test_burn_blocked_when_holder_not_compliant`, `test_burn_blocked_when_holder_suspended` |
| Mint to a non-compliant recipient | **Rejected** with `RecipientNotCompliant` (#8) | Minting is the only way new supply enters circulation; leaving it ungated would let tokens reach an address no `transfer` could ever reach. `mint_batch` applies the same check per recipient. | `test_mint_to_noncompliant_fails`, `test_mint_batch_reverts_entirely_on_noncompliant_recipient` |
