# Registry Contract

A canonical on-chain index of every tokenized asset on the platform. Each issuer
registers their asset-token contract; the registry assigns an incrementing id
and reports total value locked (TVL).

- Testnet: `CBX5SMLTXX6JP4HA5GQIO2V6QM7WCUGL2GZ6D4U773HMRI6RXISKPUR3`

## `AssetEntry`

| Field           | Type      | Meaning                          |
|-----------------|-----------|----------------------------------|
| `id`            | `u64`     | Registry id (1-based)            |
| `token_contract`| `Address` | The asset-token contract         |
| `issuer`        | `Address` | Who registered it                |
| `name`          | `String`  | Asset name                       |
| `asset_type`    | `String`  | `real_estate` / `invoice` / ...  |
| `valuation`     | `i128`    | USD cents                        |
| `created_at`    | `u32`     | Ledger sequence at registration  |
| `active`        | `bool`    | Counted in TVL while true        |

## Functions

- `initialize(admin)` — sets admin. Once only. `AlreadyInitialized (#1)`.
- `register_asset(issuer, token_contract, name, asset_type, valuation) -> u64` —
  issuer auth; assigns and returns the id. `InvalidValuation (#5)` if negative,
  `InvalidInput (#7)` if `name` is empty or `asset_type` is not one of the
  canonical [asset types](#asset-types).
- `get_asset(asset_id) -> AssetEntry` — `AssetNotFound (#4)`.
- `get_assets_by_issuer(issuer) -> Vec<AssetEntry>`
- `get_assets_by_type(asset_type) -> Vec<AssetEntry>` — see
  [matching rules](#asset-types) below; the match is byte-exact.
- `get_all_assets(start_id, limit) -> Vec<AssetEntry>` — paginated.
- `deactivate_asset(admin, asset_id)` — admin auth; sets `active=false`;
  removes the asset's valuation from `total_value_locked()`. No-op (no event)
  if the asset is already inactive.
- `reactivate_asset(admin, asset_id)` — admin auth; sets `active=true`;
  restores the asset's valuation to `total_value_locked()`. No-op (no event)
  if the asset is already active. See [Deactivation is reversible](#deactivation-is-reversible).
- `total_value_locked() -> i128` — sum of `valuation` over active assets.
  O(1): maintained as a running total updated on register, deactivate and
  reactivate, never recomputed by iterating the registry.
- `asset_count() -> u64` — total registrations, active or not.
- `active_count() -> u64` — registrations currently active.
- `get_admin() -> Address`

## Asset types

`asset_type` is validated against a fixed, canonical list at registration
time (`VALID_ASSET_TYPES` in `contracts/registry/src/lib.rs`):

`real_estate`, `invoice`, `commodity`, `bond`, `equity`, `fund`

Any other value — including a near-miss like a typo, different casing, or
extra whitespace — is rejected with `InvalidInput (#7)`. This prevents a
typo from silently creating a category that no filter will ever match.

**Matching rule for `get_assets_by_type`:** matching is byte-exact
(case-sensitive and whitespace-sensitive). The per-type index key is the
`asset_type` string exactly as it was stored at registration, so
`"real_estate"`, `"Real_Estate"` and `"real_estate "` are distinct keys.
Because `register_asset` only ever accepts the canonical lowercase strings
above, callers should always query with one of those exact strings — passing
a differently-cased or padded value returns an empty list, not an error.

## Deactivation is reversible

`deactivate_asset` is **not** a one-way operation. An asset can be
deactivated by mistake (wrong id, premature admin action), and forcing a
re-registration to "undo" it would assign a new id, breaking any external
references, the issuer index, and the type index that point at the original
id. Instead, `reactivate_asset` restores the same `AssetEntry` in place: it
flips `active` back to `true` and adds the valuation back into both
`active_count()` and `total_value_locked()`. Both operations are admin-only
and idempotent (repeating either one when already in that state is a no-op).

## Errors

| Code | Name               | Cause                                          |
|------|--------------------|-------------------------------------------------|
| 1    | AlreadyInitialized | double init                                    |
| 2    | NotInitialized     | used before init                               |
| 3    | Unauthorized       | non-admin deactivate/reactivate                |
| 4    | AssetNotFound      | unknown id                                     |
| 5    | InvalidValuation   | negative valuation                             |
| 6    | Overflow           | TVL running total overflowed i128              |
| 7    | InvalidInput       | empty name, or `asset_type` not in the canonical list |

## Events

| Topic       | Data              | When                |
|-------------|-------------------|---------------------|
| `init`      | admin             | initialize          |
| `register`  | (issuer) → id     | asset registered    |
| `deactvate` | asset_id          | asset deactivated   |
| `reactvate` | asset_id          | asset reactivated   |

## Storage / TTL

Listing of the contract `DataKey` variants and their storage behaviour.

| Key | Payload | Storage | TTL / Notes |
|-----|---------|---------|-------------|
| `Admin` | - | instance | - |
| `Counter` | - | instance | monotonic id counter |
| `Ids` | - | - | legacy key, no longer written; kept for read-compat with old deployments |
| `Asset` | u64 | persistent | extended on read/write |
| `ActiveCount` | - | instance | count of currently-active assets |
| `IssuerIndex` | Address | persistent | ids registered by that issuer; extended on read/write |
| `TypeIndex` | String | persistent | ids of that exact `asset_type` string; extended on read/write |
| `TotalValuation` | - | instance | running TVL total; O(1) read, updated on register/deactivate/reactivate |

## Security considerations

- Registration requires the **issuer** to authorize; anyone can register their
  own asset, but only the admin can deactivate or reactivate entries.
- TVL is a running total over active entries, updated incrementally on
  register, deactivate and reactivate, so it reflects the current state
  immediately without iterating the registry.
