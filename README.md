# Kwenta referrals via Axiom

The `AxiomKwentaIncentives` contract allows traders on Kwenta to claim incentives based on the total amount of `executorFee` paid in trades they have made. Users can use Axiom to prove in ZK that they paid a total sum of `executorFee` to the protocol. We identify each claimed trade with a `claimId` which is a monotone increasing identifier of all Ethereum receipts; to prevent double claiming, we enforce that claims must be made in increasing order of `claimId`.

The ZK-proven results via Axiom provided to `AxiomKwentaIncentives` via callback are:

- `uint256 startClaimId` -- the smallest `claimId` in the claimed batch
- `uint256 endClaimId` -- the largest `claimId` in the claimed batch
- `uint128 claimerAccount` -- the `accountId` for all claims in this batch
- `uint256 totalValue` -- the total value of ETH-denominated `executorFee` in this batch.

## Development

To set up the development environment, run:

```
forge install
npm install   # or `yarn install` or `pnpm install`
```

To run tests, create a `.env` file, set `BASE_PROVIDER_URI`, and then run

```
forge test
```
