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
  issuer auth; assigns and returns the id. `InvalidValuation (#5)` if negative.
  Rejects `token_contract` values already registered under another id with
  `DuplicateAsset (#8)` — see "Duplicate registration" below.
- `get_asset(asset_id) -> AssetEntry` — `AssetNotFound (#4)`.
- `get_assets_by_issuer(issuer) -> Vec<AssetEntry>`
- `get_assets_by_type(asset_type) -> Vec<AssetEntry>`
- `get_all_assets(start_id, limit) -> Vec<AssetEntry>` — returns ids
  `[start_id, start_id + limit)`, capped at the current counter and at
  `MAX_PAGE_SIZE` (100) regardless of the requested `limit`. Page through the
  full registry by calling again with `start_id + <count returned>`. A small
  registry that fits in one page keeps working with a single call
  (`start_id = 1`, a large `limit`).
- `deactivate_asset(admin, asset_id)` — admin auth; sets `active=false`.
- `total_value_locked() -> i128` — sum of `valuation` over active assets.
- `asset_count() -> u64`
- `get_admin() -> Address`

### Duplicate registration (issue #308)

`register_asset` maintains a reverse index from `token_contract` to its
assigned asset id. If the same token contract address is registered a second
time — under any issuer, name, or asset type — the call reverts with
`DuplicateAsset (#8)` before any state changes. This is intentional: allowing
the same token contract under two registry ids would let `total_value_locked`
and `get_all_assets` double-count it, inflating the TVL figure shown on the
landing page and producing duplicate entries on the explore page. Covered by
`test_duplicate_token_contract_registration_rejected` in
`contracts/registry/src/test.rs`.

### Pagination and max page size (issue #310)

`get_all_assets` always enforces `MAX_PAGE_SIZE = 100` as an upper bound on
the number of entries returned in one call, independent of the `limit`
argument passed in. This keeps per-call cost bounded as the registry grows,
while existing callers that pass a large `limit` to fetch everything in one
shot keep working unchanged as long as the registry is smaller than the cap.
The final page of a paginated walk is partial once fewer than `limit` assets
remain; see `test_get_all_assets_final_partial_page` and
`test_get_all_assets_enforces_max_page_size` in
`contracts/registry/src/test.rs`.

### Deactivation and TVL (issue #306)

`deactivate_asset` is intentionally **exclusive**: as soon as an asset is
deactivated, its `valuation` is subtracted from `TotalValuation` in the same
call, and it is excluded from every subsequent `total_value_locked()` read.
It is never re-added implicitly — an asset must be re-registered (as a new
id) to count again. This is the correct behavior for a headline TVL figure:
a deactivated asset (e.g. delisted, fraudulent, or redeemed) should not
inflate the number shown on the landing page. The web app's TVL display
reads `total_value_locked()` directly, so it reflects this automatically.
Covered by `test_deactivate_excludes_from_tvl` and `test_tvl_sums_only_active`
in `contracts/registry/src/test.rs`.

## Errors

| Code | Name               | Cause                          |
|------|--------------------|--------------------------------|
| 1    | AlreadyInitialized | double init                    |
| 2    | NotInitialized     | used before init               |
| 3    | Unauthorized       | non-admin deactivation         |
| 4    | AssetNotFound      | unknown id                     |
| 5    | InvalidValuation   | negative valuation             |
| 8    | DuplicateAsset     | token_contract already registered |

## Events

| Topic       | Data              | When              |
|-------------|-------------------|-------------------|
| `init`      | admin             | initialize        |
| `register`  | (issuer) → id     | asset registered  |
| `deactvate` | asset_id          | asset deactivated |

## Storage / TTL

Listing of the contract `DataKey` variants and their storage behaviour.

| Key | Payload | Storage | TTL / Notes |
|-----|---------|---------|-------------|
| `Admin` | - | instance | - |
| `Counter` | - | instance | - |
| `Ids` | - | unknown | - |
| `Asset` | u64 | persistent | extended via instance() |

## Security considerations

- Registration requires the **issuer** to authorize; anyone can register their
  own asset, but only the admin can deactivate entries.
- TVL is computed from active entries only, so deactivating an asset removes it
  from platform totals immediately.
