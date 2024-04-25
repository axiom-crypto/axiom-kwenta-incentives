use std::{fmt::Debug, str::FromStr};

use axiom_circuit::{axiom_eth::halo2curves::ff::Field, input::{flatten::FixLenVec, raw_input::RawInput}};

use axiom_sdk::{
    axiom::{AxiomAPI, AxiomComputeFn, AxiomComputeInput, AxiomResult}, 
    ethers::{
        abi::Address,
        types::H256, 
    },
    halo2_base::{
        gates::{GateInstructions, RangeInstructions}, 
        AssignedValue,
        QuantumCell::Constant,
    },
    Fr, 
    HiLo
};

use lazy_static::lazy_static;

const MAX_INPUTS: usize = 16;

lazy_static! {
    static ref CONDITIONAL_ORDER_EXECUTED_SCHEMA: Option<H256> = H256::from_str("0x3f4c4edf80aee6ea6f4fb3aed498a467e30ed1482bc06539ffe892cf7304e334").ok();
    static ref KWENTA_SMART_MARGIN_V3_ADDR: Option<Address> = Address::from_str("0xe331a7eeC851Ba702aA8BF43070a178451d6D28E").ok();
}

#[AxiomComputeInput]
pub struct ClaimInput {
    pub block_numbers: FixLenVec<usize, MAX_INPUTS>,
    pub tx_idxs: FixLenVec<usize, MAX_INPUTS>,
    pub log_idxs: FixLenVec<usize, MAX_INPUTS>,
    pub executor_fees: FixLenVec<usize, MAX_INPUTS>,
    pub num_claims: usize,
}

impl AxiomComputeFn for ClaimInput {
    fn compute(
        api: &mut AxiomAPI,
        assigned_inputs: ClaimCircuitInput<AssignedValue<Fr>>,
    ) -> Vec<AxiomResult> {
        let gate = api.range.gate();
        let range = api.range;
        let ctx = api.ctx();

        let zero = ctx.load_constant(Fr::ZERO);
        let one = ctx.load_constant(Fr::ONE);
        let two = ctx.load_constant(Fr::from(2u64));
        let nine = ctx.load_constant(Fr::from(9u64));
        let modulus = gate.pow_of_two()[128];
        let modulus = ctx.load_constant(modulus);
        let two_pow_64 = gate.pow_of_two[64];
        let kwenta_contract = ctx.load_constant(KWENTA_SMART_MARGIN_V3_ADDR.unwrap().convert());

        // Check num claims
        api.range.check_less_than(
            api.ctx(), 
            Constant(Fr::ZERO),
            assigned_inputs.num_claims, 
            16
        );

        api.range.check_less_than(
            api.ctx(),
            assigned_inputs.num_claims,
            Constant(Fr::from((MAX_INPUTS + 1) as u64)),
            16
        );

        // Generate claim IDs
        let mut claim_ids = vec![];
        let mut in_range = vec![];
        for i in 0..MAX_INPUTS {
            let id_1 = gate.mul_add(
                api.ctx(),
                assigned_inputs.block_numbers[i],
                Constant(two_pow_64),
                assigned_inputs.tx_idxs[i]
            );
            let claim_id = gate.mul_add(
                api.ctx(),
                id_1,
                Constant(two_pow_64),
                assigned_inputs.log_idxs[i]
            );
            let is_in_range = api.range.is_less_than(
                api.ctx(),
                Constant(Fr::from(i as u64)),
                assigned_inputs.num_claims,
                64
            );
            in_range.push(is_in_range);
            let claim_id_or_zero = gate.mul(api.ctx(), claim_id, is_in_range);
            claim_ids.push(claim_id_or_zero);
        }

        // Check claim IDs are in increasing order
        for i in 1..MAX_INPUTS {
            let is_less_than = api.range.is_less_than(
                api.ctx(),
                claim_ids[i - 1],
                claim_ids[i],
                192
            );
            let claim_id_is_zero = gate.is_zero(
                api.ctx(),
                claim_ids[i]
            );
            let is_less_than_or_zero = gate.or(
                api.ctx(),
                is_less_than,
                claim_id_is_zero
            );
            api.ctx().constrain_equal(
                &is_less_than_or_zero,
                &one
            );
        }

        // Check that:
        //   - all valid events are emitted from the correct contract
        //   - all valid events have the same account_id
        let mut logged_account_id = api.ctx().load_constant(Fr::ZERO);
        for i in 0..MAX_INPUTS {
            // Validate event contract is equivalent to Kwenta contract
            let event_contract = api
                .get_receipt(assigned_inputs.block_numbers[i], assigned_inputs.tx_idxs[i])
                .log(assigned_inputs.log_idxs[i])
                .address();
            let event_contract = api.from_hi_lo(event_contract);
            let event_contract_is_equal = gate.is_equal(api.ctx(), event_contract, kwenta_contract);
            let is_out_of_range = gate.is_zero(api.ctx(), in_range[i]);
            let event_contract_or_zero = gate.or(api.ctx(), event_contract_is_equal, is_out_of_range);
            api.ctx().constrain_equal(&event_contract_or_zero, &one);

            // Check the account
            let account_id_market_id_hilo = api
                .get_receipt(assigned_inputs.block_numbers[i], assigned_inputs.tx_idxs[i])
                .log(assigned_inputs.log_idxs[i])
                .data(two, *CONDITIONAL_ORDER_EXECUTED_SCHEMA);
            let account_id = account_id_market_id_hilo.hi();
            if i == 0 {
                logged_account_id = account_id;
            } else {
                let account_id_is_equal = gate.is_equal(api.ctx(), account_id, logged_account_id);
                let is_out_of_range = gate.is_zero(api.ctx(), in_range[i]);
                let account_id_or_zero = gate.or(api.ctx(), account_id_is_equal, is_out_of_range);
                api.ctx().constrain_equal(&account_id_or_zero, &one);
            }
        }

        // Sum executor fees
        let mut total_executor_fee = HiLo::from_hi_lo([zero, zero]);
        for i in 0..MAX_INPUTS {
            let executor_fee = api
                .get_receipt(assigned_inputs.block_numbers[i], assigned_inputs.tx_idxs[i])
                .log(assigned_inputs.log_idxs[i])
                .data(nine, *CONDITIONAL_ORDER_EXECUTED_SCHEMA);
            let executor_fee = {
                let executor_fee_hi = gate.mul(api.ctx(), executor_fee.hi(), in_range[i]);
                let executor_fee_lo = gate.mul(api.ctx(), executor_fee.lo(), in_range[i]);
                HiLo::from_hi_lo([executor_fee_hi, executor_fee_lo])
            };
            total_executor_fee = {
                let sum = gate.add(api.ctx(), total_executor_fee.lo(), executor_fee.lo());
                let carry = range.is_less_than(api.ctx(), sum, modulus, 129);
                let carry = range.gate().not(api.ctx(), carry);
                let lo = gate.sub_mul(api.ctx(), sum, carry, modulus);
                let hi = gate.add(api.ctx(), carry, total_executor_fee.hi());
                HiLo::from_hi_lo([hi, lo])
            };
        }
        
        // Output [start_claim_id, end_claim_id, incentive_id, total_value]
        let first_claim_id = claim_ids[0];
        let last_idx = gate.sub(api.ctx(), assigned_inputs.num_claims, one);
        let last_claim_id = gate.select_from_idx(
            api.ctx(),
            claim_ids,
            last_idx,
        );
        vec![
            first_claim_id.into(),
            last_claim_id.into(),
            
            total_executor_fee.into(),
        ]
    }
}