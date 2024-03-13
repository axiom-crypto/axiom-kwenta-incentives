import {
  sum,
  div,
  addToCallback,
  CircuitValue,
  constant,
  witness,
  mul,
  add,
  checkLessThan,
  getReceipt,
  checkEqual,
  mulAdd,
  log,
  selectFromIdx,
  sub,
  isLessThan,
  isZero,
  or,
  mod,
  getHeader,
  rangeCheck,
  isEqual,
  and
} from "@axiom-crypto/client";

/// For type safety, define the input types to your circuit here.
/// These should be the _variable_ inputs to your circuit. Constants can be hard-coded into the circuit itself.
export interface CircuitInputs {
  blockNumbers: CircuitValue[];
  txIdxs: CircuitValue[];
  logIdxs: CircuitValue[];
  numClaims: CircuitValue;
}

export const defaultInputs = {
  "blockNumbers": [
    11568267, 11568267, 11568267, 11568267, 11568267, 11568267, 11568267, 11568267, 11568267, 11568267
  ],
  "txIdxs": [3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
  "logIdxs": [2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
  "numClaims": 1
}

export const circuit = async ({
  blockNumbers,
  txIdxs,
  logIdxs,
  numClaims
}: CircuitInputs) => {

  const MAX_CLAIMS = 10;
  const KWENTA_SMART_MARGIN_V3_ADDR = "0xe331a7eeC851Ba702aA8BF43070a178451d6D28E";

  let numClaimsVal = Number(numClaims.value());
  if (numClaimsVal > MAX_CLAIMS) {
    throw new Error("Too many claims");
  }

  checkLessThan(0, numClaims)
  checkLessThan(numClaims, MAX_CLAIMS + 1)

  if (blockNumbers.length !== MAX_CLAIMS || txIdxs.length !== MAX_CLAIMS || logIdxs.length !== MAX_CLAIMS) {
    throw new Error("Incorrect number of claims (make sure every array has `MAX_CLAIMS` claims)");
  }

  let claimIds: CircuitValue[] = [];
  let inRange: CircuitValue[] = [];
  for (let i = 0; i < MAX_CLAIMS; i++) {
    const id_1 = mulAdd(blockNumbers[i], BigInt(2 ** 64), txIdxs[i]);
    const id = mulAdd(id_1, BigInt(2 ** 64), logIdxs[i]);
    const isInRange = isLessThan(i, numClaims, "20");
    inRange.push(isInRange);
    const idOrZero = mul(id, isInRange);
    claimIds.push(idOrZero);
  }

  for (let i = 1; i < MAX_CLAIMS; i++) {
    const isLess = isLessThan(claimIds[i - 1], claimIds[i]);
    const isLessOrNotInRange = or(isLess, isZero(claimIds[i]));
    checkEqual(isLessOrNotInRange, 1);
  }

  // Custom logic for each incentives program

  const CONDITIONAL_ORDER_EXECUTED_SCHEMA = "0x3f4c4edf80aee6ea6f4fb3aed498a467e30ed1482bc06539ffe892cf7304e334";
  //event: https://basescan.org/tx/0xfe72edad57c1421c91a6526e4582fb6497298659177ef7e14ce2c5edac05332c#eventlog
  //     event ConditionalOrderExecuted(IPerpsMarketProxy.Data order, uint256 synthetixFees, uint256 executorFee);
  /*
      struct Data {
        /// @dev Time at which the Settlement time is open.
        uint256 settlementTime;
        /// @dev Order request details.
        OrderCommitmentRequest request;
    }

    struct OrderCommitmentRequest {
        /// @dev Order market id.
        uint128 marketId;
        /// @dev Order account id.
        uint128 accountId;
        /// @dev Order size delta (of asset units expressed in decimal 18 digits). It can be positive or negative.
        int128 sizeDelta;
        /// @dev Settlement strategy used for the order.
        uint128 settlementStrategyId;
        /// @dev Acceptable price set at submission.
        uint256 acceptablePrice;
        /// @dev An optional code provided by frontends to assist with tracking the source of volume and fees.
        bytes32 trackingCode;
        /// @dev Referrer address to send the referrer fees to.
        address referrer;
    }
   */
  let claimerAccount = constant(0);
  let totalValue = witness(0);
  for (let i = 0; i < MAX_CLAIMS; i++) {
    // check that the event was emitted by the correct contract
    const emitter = (await getReceipt(blockNumbers[i], txIdxs[i]).log(logIdxs[i]).address()).toCircuitValue();
    checkEqual(KWENTA_SMART_MARGIN_V3_ADDR, emitter);

    // check that all events have the same accountId
    let thisClaimerAccount = (await getReceipt(blockNumbers[i], txIdxs[i]).log(logIdxs[i]).data(2, CONDITIONAL_ORDER_EXECUTED_SCHEMA)).toCircuitValue();
    if (i === 0) {
      claimerAccount = thisClaimerAccount;
    } else {
      checkEqual(claimerAccount, thisClaimerAccount);
    }

    // sum up the executorFee 
    let executorFee = (await getReceipt(blockNumbers[i], txIdxs[i]).log(logIdxs[i]).data(9)).toCircuitValue();

    /* Replace with the line below to use the synthetixFee
    let synthetixFee = (await getReceipt(blockNumbers[i], txIdxs[i]).log(logIdxs[i]).data(8)).toCircuitValue();
    */

    let amountOrZero = mul(executorFee, inRange[i]);
    totalValue = add(totalValue, amountOrZero);
  }

  const lastClaimId = selectFromIdx(claimIds, sub(numClaims, constant(1)));

  addToCallback(claimIds[0]);
  addToCallback(lastClaimId);
  addToCallback(claimerAccount);
  addToCallback(totalValue);
};
