# Valuation Unit Convention

**All valuations are stored as USD CENTS in i128, not dollars.**

## Why Cents?

Avoiding floating-point arithmetic. 1 cent = 1 unit.

## Convention

- USD 100.50 = 10,050 (cents)
- USD 1.99 = 199 (cents)
- USD 0.01 = 1 (cent)

## Conversion

```rust
// To cents: multiply by 100
dollars_to_cents(100.5) = 10_050

// From cents: divide by 100
cents_to_dollars(10_050) = 100.5
```

## In Registry Contract

`register_asset(..., 50000)` stores **$500.00** (50,000 cents)

All queries return cents. Convert on client side if needed.

## Token and Registry Consistency

The asset token and registry store separate valuation fields. Registering an
asset copies the supplied valuation into the registry, but the registry does
not observe `asset-token::update_valuation`. Updating the token metadata can
therefore leave the registry entry and TVL stale. Divergence is possible until
the registry gains a valuation-update workflow; clients and operators must
verify both records and coordinate any separate registry correction.

## In Documentation

Every valuation reference must state: **"in USD cents"**
