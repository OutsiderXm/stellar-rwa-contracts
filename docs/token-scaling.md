# Token Amount Scaling Convention

All token amounts in these contracts are stored as **integers in each token's own decimal base**.

## Convention

- Asset token balances: scaled by asset's decimals
- Payment token amounts: scaled by payment token's decimals
- Mixing scales without conversion causes silent errors

## Example: USDC (6 decimals) → Asset (2 decimals)

```
USDC amount: 100.50 (human readable)
= 100_500_000 stroops (6 decimals)
= 10_050 in asset base (2 decimals, after conversion)

Conversion: usdc_amount / (10 ^ (6 - 2)) = asset_amount
```

## In Dividend Contract

When claiming from a distribution:
- Snapshot balances are in **asset decimals**
- Payment tokens are in **payment decimals**
- Claim formula: `total_amount * snapshot_balance / snapshot_supply`

All amounts must be in **consistent base** before arithmetic.

## Safety

Always convert between scales explicitly:
```rust
fn convert_amount(amount: i128, from_decimals: u32, to_decimals: u32) -> i128 {
    if from_decimals > to_decimals {
        amount / 10_i128.pow(from_decimals - to_decimals)
    } else {
        amount * 10_i128.pow(to_decimals - from_decimals)
    }
}
```
