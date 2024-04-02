// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import { AxiomIncentives } from "@axiom-crypto/axiom-incentives/AxiomIncentives.sol";

contract AxiomKwentaIncentives is AxiomIncentives {
    /// @dev The address of Synthetix CoreProxy on Base
    address public immutable SYNTHETIX_CORE;

    /// @notice Emitted when a claim is made.
    /// @param claimerAccount The ID of the referrer.
    /// @param startClaimId The ID of the first claim in the claim batch.
    /// @param endClaimId The ID of the last claim in the claim batch.
    /// @param totalValue The total value of the claim batch.
    event ClaimBatch(uint128 indexed claimerAccount, uint256 startClaimId, uint256 endClaimId, uint256 totalValue);

    /// @notice Construct a new AxiomKwentaIncentives contract.
    /// @param  _axiomV2QueryAddress The address of the AxiomV2Query contract.
    /// @param  incentivesQuerySchemas A list containing valid querySchemas for incentives.
    /// @param  synthetixCore The address of the Synthetix CoreProxy contract.
    constructor(address _axiomV2QueryAddress, bytes32[] memory incentivesQuerySchemas, address synthetixCore)
        AxiomIncentives(_axiomV2QueryAddress, incentivesQuerySchemas)
    {
        SYNTHETIX_CORE = synthetixCore;
    }

    /// @notice Claim incentive rewards
    /// @param startClaimId The ID of the first claim in the claim batch.
    /// @param endClaimId The ID of the last claim in the claim batch.
    /// @param claimerAccount The address of the claimer.
    /// @param claimerAddress The address of the claimer.
    /// @param totalValue The total value of the claim batch.
    function _claim(
        uint256 startClaimId,
        uint256 endClaimId,
        uint128 claimerAccount,
        address claimerAddress,
        uint256 totalValue
    ) internal {
        require(claimerAccount != 0, "claimerAccount cannot be zero");

        // find the address of the account owner in Synthetix v3
        (, bytes memory data) = SYNTHETIX_CORE.call(abi.encodeWithSignature("getAccountOwner(uint128)", claimerAccount));
        address claimer = abi.decode(data, (address));
        require(claimer == claimerAddress, "Cannot claim for another address");

        // TODO: Actually send funds to claimer
        emit ClaimBatch(claimerAccount, startClaimId, endClaimId, totalValue);
    }

    /// @inheritdoc AxiomIncentives
    function _validateClaim(
        bytes32, // querySchema
        address, // caller
        uint256, // startClaimId
        uint256, // endClaimId
        uint256, // incentiveId
        uint256 // totalValue
    ) internal pure override { }

    /// @inheritdoc AxiomIncentives
    function _sendClaimRewards(
        bytes32, // querySchema
        address caller,
        uint256 startClaimId,
        uint256 endClaimId,
        uint256 incentiveId,
        uint256 totalValue
    ) internal override {
        uint128 claimerAccount = uint128(incentiveId);
        _claim(startClaimId, endClaimId, claimerAccount, caller, totalValue);
        emit ClaimBatch(claimerAccount, startClaimId, endClaimId, totalValue);
    }
}
