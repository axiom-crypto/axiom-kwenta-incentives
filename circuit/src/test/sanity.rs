use std::str::FromStr;

use axiom_circuit::{input::flatten::FixLenVec, types::{AxiomCircuitParams, AxiomV2DataAndResults}, utils::get_provider};
use axiom_sdk::{axiom::AxiomCompute, ethers::types::H256, halo2_base::gates::circuit::BaseCircuitParams};

use super::utils::pad_input;

use crate::{claim::{ClaimInput, MAX_INPUTS}, test::utils::calculate_claim_id};

fn get_circuit_input(
    block_number: Vec<usize>,
    tx_idx: Vec<usize>,
    log_idx: Vec<usize>,
) -> ClaimInput {
    let (block_numbers, tx_idxs, log_idxs) = pad_input(block_number.clone(), tx_idx, log_idx, MAX_INPUTS);
    ClaimInput {
        block_numbers: FixLenVec::new(block_numbers).unwrap(),
        tx_idxs: FixLenVec::new(tx_idxs).unwrap(),
        log_idxs: FixLenVec::new(log_idxs).unwrap(),
        num_claims: block_number.len(),
    }
}

fn run_circuit(
    block_number: Vec<usize>,
    tx_idx: Vec<usize>,
    log_idx: Vec<usize>,
) -> AxiomV2DataAndResults {
    let params = BaseCircuitParams {
        k: 12,
        num_advice_per_phase: vec![4],
        num_fixed: 1,
        num_lookup_advice_per_phase: vec![1],
        lookup_bits: Some(11),
        num_instance_columns: 1,
    };

    let provider = get_provider();
    let input = get_circuit_input(block_number, tx_idx, log_idx);
    let compute = AxiomCompute::<ClaimInput>::new()
        .use_params(AxiomCircuitParams::Base(params))
        .use_provider(provider)
        .use_inputs(input);
    compute.mock();
    let circuit = compute.circuit();
    circuit.scaffold_output()
}

#[test]
fn test_one_claim() {
    let block_numbers = vec![13571616];
    let tx_idxs = vec![40];
    let log_idxs = vec![2];
    let output = run_circuit(
        block_numbers.clone(),
        tx_idxs.clone(),
        log_idxs.clone(),
    );
    let start_claim_id = calculate_claim_id(block_numbers[0], tx_idxs[0], log_idxs[0]);
    assert_eq!(output.compute_results[0], start_claim_id);
    assert_eq!(output.compute_results[1], start_claim_id);
    assert_eq!(output.compute_results[2], H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap());
    assert_eq!(output.compute_results[3], H256::from_str("0x00000000000000000000000000000000000000000000000001823a7246d8e4f8").unwrap());
}

#[test]
fn test_two_claims() {
    let block_numbers = vec![13571616, 13572321];
    let tx_idxs = vec![40, 28];
    let log_idxs = vec![2, 2];
    let output = run_circuit(
        block_numbers.clone(),
        tx_idxs.clone(),
        log_idxs.clone(),
    );
    let start_claim_id = calculate_claim_id(block_numbers[0], tx_idxs[0], log_idxs[0]);
    let end_claim_id = calculate_claim_id(block_numbers[1], tx_idxs[1], log_idxs[1]);
    assert_eq!(output.compute_results[0], start_claim_id);
    assert_eq!(output.compute_results[1], end_claim_id);
    assert_eq!(output.compute_results[2], H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap());
    // sum:
    // 00000000000000000000000000000000000000000000000001823a7246d8e4f8
    // 00000000000000000000000000000000000000000000000001bbcdb4dceb30b1
    assert_eq!(output.compute_results[3], H256::from_str("0x000000000000000000000000000000000000000000000000033e082723c415a9").unwrap());
}

#[test]
fn test_multiple_claims() {
    let block_numbers = vec![13571616, 13572321, 13572709, 13582226, 13584316];
    let tx_idxs = vec![40, 28, 26, 30, 40];
    let log_idxs = vec![2, 2, 2, 2, 2];
    let output = run_circuit(
        block_numbers.clone(),
        tx_idxs.clone(),
        log_idxs.clone(),
    );
    let start_claim_id = calculate_claim_id(block_numbers[0], tx_idxs[0], log_idxs[0]);
    let end_claim_id = calculate_claim_id(block_numbers[4], tx_idxs[4], log_idxs[4]);
    assert_eq!(output.compute_results[0], start_claim_id);
    assert_eq!(output.compute_results[1], end_claim_id);
    assert_eq!(output.compute_results[2], H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap());
    // sum:
    // 00000000000000000000000000000000000000000000000001823a7246d8e4f8
    // 00000000000000000000000000000000000000000000000001bbcdb4dceb30b1
    // 000000000000000000000000000000000000000000000000017a5097950bd46a
    // 00000000000000000000000000000000000000000000000003c11eb937b0faf4
    // 00000000000000000000000000000000000000000000000003adb153947d1aac
    assert_eq!(output.compute_results[3], H256::from_str("0x0000000000000000000000000000000000000000000000000c2728cb84fdffb3").unwrap());
}

#[test]
#[should_panic]
fn test_fail_same_claims() {
    let block_numbers = vec![13572321, 13572321];
    let tx_idxs = vec![28, 28];
    let log_idxs = vec![2, 2];
    let _output = run_circuit(
        block_numbers.clone(),
        tx_idxs.clone(),
        log_idxs.clone(),
    );
}

#[test]
#[should_panic]
fn test_fail_non_increasing_claims() {
    let block_numbers = vec![13571616, 13572321, 13582226, 13572709, 13584316];
    let tx_idxs = vec![40, 28, 30, 26, 40];
    let log_idxs = vec![2, 2, 2, 2, 2];
    let _output = run_circuit(
        block_numbers.clone(),
        tx_idxs.clone(),
        log_idxs.clone(),
    );
}

#[test]
#[should_panic]
fn test_fail_invalid_claim() {
    // claim[2] is invalid
    let block_numbers = vec![13571616, 13572321, 13633667, 13572709, 13584316];
    let tx_idxs = vec![40, 28, 44, 26, 40];
    let log_idxs = vec![2, 2, 1, 2, 2];
    let _output = run_circuit(
        block_numbers.clone(),
        tx_idxs.clone(),
        log_idxs.clone(),
    );
}

// WIP: Currently all `ConditionalOrderExecuted` events are from account ID 0.
// #[test]
// fn test_fail_different_account_ids() {}
