// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import { AxiomV2Client } from "@axiom-crypto/v2-periphery/client/AxiomV2Client.sol";

contract AxiomKwentaIncentives is AxiomV2Client {
    /// @dev The unique identifier of the claim circuit accepted by this contract.
    bytes32 public immutable CLAIM_QUERY_SCHEMA;

    /// @dev The address of Synthetix CoreProxy on Base
    address public immutable SYNTHETIX_CORE;

    /// @dev `lastClaimedId[claimerAccount]` is the latest `claimId` at which `claimerAccount` claimed an incentive for an event.
    /// @dev `claimId` = `blockNumber` * 2^128 + `txIdx` * 2^64 + `logIdx`
    mapping(uint128 => uint256) public lastClaimedId;

    /// @notice Emitted when a claim is made.
    /// @param claimerAccount The ID of the referrer.
    /// @param startClaimId The ID of the first claim in the claim batch.
    /// @param endClaimId The ID of the last claim in the claim batch.
    /// @param totalValue The total value of the claim batch.
    event ClaimBatch(uint128 indexed claimerAccount, uint256 startClaimId, uint256 endClaimId, uint256 totalValue);

    /// @notice Construct a new AxiomNonceIncrementor contract.
    /// @param  _axiomV2QueryAddress The address of the AxiomV2Query contract.
    /// @param  synthetixCore The address of the Synthetix CoreProxy contract.
    /// @param  claimQuerySchema The unique identifier of the query schema accepted by this contract.
    constructor(address _axiomV2QueryAddress, address synthetixCore, bytes32 claimQuerySchema)
        AxiomV2Client(_axiomV2QueryAddress)
    {
        SYNTHETIX_CORE = synthetixCore;
        CLAIM_QUERY_SCHEMA = claimQuerySchema;
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
        require(lastClaimedId[claimerAccount] < startClaimId, "Already claimed");
        require(claimerAccount != 0, "Claimer cannot be zero");

        // find the address of the account owner in Synthetix v3
        (, bytes memory data) = SYNTHETIX_CORE.call(abi.encodeWithSignature("getAccountOwner(uint128)", claimerAccount));
        address claimer = abi.decode(data, (address));
        require(claimer == claimerAddress, "Cannot claim for another address");

        lastClaimedId[claimerAccount] = endClaimId;

        // TODO: Actually send funds to claimer
        emit ClaimBatch(claimerAccount, startClaimId, endClaimId, totalValue);
    }

    /// @inheritdoc AxiomV2Client
    function _validateAxiomV2Call(
        AxiomCallbackType, // callbackType,
        uint64 sourceChainId,
        address, // caller,
        bytes32 querySchema,
        uint256, // queryId,
        bytes calldata // extraData
    ) internal view override {
        require(sourceChainId == block.chainid, "Source chain ID does not match");
        require(querySchema == CLAIM_QUERY_SCHEMA, "Invalid query schema");
    }

    /// @inheritdoc AxiomV2Client
    function _axiomV2Callback(
        uint64, // sourceChainId,
        address caller,
        bytes32, // querySchema
        uint256, // queryId,
        bytes32[] calldata axiomResults,
        bytes calldata // extraData
    ) internal override {
        uint256 startClaimId = uint256(axiomResults[0]);
        uint256 endClaimId = uint256(axiomResults[1]);
        uint128 claimerAccount = uint128(uint256(axiomResults[2]));
        uint256 totalValue = uint256(axiomResults[3]);

        _claim(startClaimId, endClaimId, claimerAccount, caller, totalValue);
    }
}
