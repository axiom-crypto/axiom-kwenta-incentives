# Kwenta Incentives via Axiom

The `AxiomKwentaIncentives` contract allows traders on Kwenta to claim incentives based on the total amount of `executorFee` paid in trades they have made. Users can use Axiom to prove in ZK that they paid a total sum of `executorFee` to the protocol. We identify each claimed trade with a `claimId` which is a monotone increasing identifier of all Ethereum receipts; to prevent double claiming, we enforce that claims must be made in increasing order of `claimId`.

This Kwenta Incentives contract uses the [AxiomIncentives](https://github.com/axiom-crypto/axiom-incentives) system built using [Axiom](https://axiom.xyz), which allows rewarding users based on ZK-proven on-chain activity.

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
