// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/console.sol";

import "@axiom-crypto/axiom-std/AxiomTest.sol";

import { AxiomKwentaIncentives } from "../src/AxiomKwentaIncentives.sol";

contract AxiomKwentaIncentivesTest is AxiomTest {
    using Axiom for Query;

    AxiomKwentaIncentives public axiomIncentives;

    struct AxiomInput {
        uint64[] blockNumbers;
        uint64[] txIdxs;
        uint64[] logIdxs;
        uint64 numClaims;
    }

    AxiomInput public input;
    bytes32 public querySchema;
    address public synthetixCoreAddress;

    uint64 constant NUM_CLAIMS = 10;

    event ClaimBatch(uint128 indexed claimer, uint256 startClaimId, uint256 endClaimId, uint256 totalValue);

    function setUp() public {
        vm.createSelectFork("base", 11_572_365);
        axiomV2QueryAddress = 0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF;
        synthetixCoreAddress = 0x32C222A9A159782aFD7529c87FA34b96CA72C696;
        axiomVm = new AxiomVm(axiomV2QueryAddress, "base");

        uint64[] memory blockNumbers = new uint64[](NUM_CLAIMS);
        for (uint256 i = 0; i < NUM_CLAIMS; i++) {
            blockNumbers[i] = 11_568_267;
        }
        uint64[] memory txIdxs = new uint64[](NUM_CLAIMS);
        for (uint256 i = 0; i < NUM_CLAIMS; i++) {
            txIdxs[i] = 3;
        }
        uint64[] memory logIdxs = new uint64[](NUM_CLAIMS);
        for (uint256 i = 0; i < NUM_CLAIMS; i++) {
            logIdxs[i] = 2;
        }
        input = AxiomInput({ blockNumbers: blockNumbers, txIdxs: txIdxs, logIdxs: logIdxs, numClaims: 1 });

        querySchema = axiomVm.readCircuit("app/axiom/claim.circuit.ts", "aaaa");
        axiomIncentives = new AxiomKwentaIncentives(axiomV2QueryAddress, synthetixCoreAddress, querySchema);
    }

    function test_proveClaim() public {
        // set up optional parameters for the query callback and fees
        bytes memory callbackExtraData = bytes("deadbeef00000000000000000000000000000000000000000000000000000000");
        IAxiomV2Query.AxiomV2FeeData memory feeData = IAxiomV2Query.AxiomV2FeeData({
            maxFeePerGas: 35 gwei,
            callbackGasLimit: 1_000_000,
            overrideAxiomQueryFee: 0
        });

        // mock a query into Axiom
        address caller = 0x3D51B90E05D682B8195e92Cb1B212B85710B48da;
        vm.deal(caller, 1 ether);
        Query memory q =
            query(querySchema, abi.encode(input), address(axiomIncentives), callbackExtraData, feeData, caller);

        // mock the query fulfillment for the claim
        uint256 claimId = 3_936_477_275_933_384_035_914_062_235_131_111_173_364_121_602;
        vm.expectEmit();
        emit ClaimBatch(
            170_141_183_460_469_231_731_687_303_715_884_105_742, claimId, claimId, 1_000_000_000_000_000_000
        );
        bytes32[] memory results = q.prankFulfill();

        // check that claimId was updated correctly
        require(
            axiomIncentives.lastClaimedId(uint128(uint256(results[2]))) == uint256(results[1]),
            "Last claim ID not updated"
        );
    }
}
