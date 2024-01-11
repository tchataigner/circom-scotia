// Copyright (c) 2022 Nalin
// Copyright (c) Lurk Lab
// SPDX-License-Identifier: MIT
//! # Circom Scotia Circuit & Configuration
//!
//! This module provides structures to work with various files generated by Circom.

use std::{path::Path, sync::Mutex};

use anyhow::Result;
use ff::PrimeField;
use serde::{Deserialize, Serialize};

use crate::error::CircomConfigError::{LoadR1CSError, WitnessCalculatorInstantiationError};
use crate::error::ReaderError::FilenameError;
use crate::{reader::load_r1cs, witness::WitnessCalculator};

/// Represents a Circom circuit with constraints and an optional witness.
///
/// This structure holds the [`R1CS`] constraints of a circuit along with the witness values
/// that satisfy these constraints, if they are available.
#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct CircomCircuit<F: PrimeField> {
    r1cs: R1CS<F>,
    witness: Option<Vec<F>>,
}

/// Data structure to hold R1CS (Rank-1 Constraint System) information.
///
/// This includes the number of public inputs and outputs, total number of inputs, auxiliary inputs,
/// variables, and the [`Constraint`] themselves.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct R1CS<F: PrimeField> {
    pub num_pub_in: usize,
    pub num_pub_out: usize,
    pub num_inputs: usize,
    pub num_aux: usize,
    pub num_variables: usize,
    pub constraints: Vec<Constraint<F>>,
}

/// Structure representing inputs for a Circom gadget.
///
/// This structure is used to represent the inputs that are fed into a Circom circuit.
/// It consists of the name of the input and the corresponding value as a vector of field elements.
#[derive(Serialize, Deserialize)]
pub struct CircomInput<F: PrimeField> {
    pub name: String,
    pub value: Vec<F>,
}

impl<F: PrimeField> CircomInput<F> {
    pub fn new(name: String, value: Vec<F>) -> Self {
        Self { name, value }
    }
}

/// Type alias for representing a single constraint in an R1CS.
///
/// A constraint is represented as a tuple of three vectors. Each vector contains pairs
/// of indices and field elements. These vectors correspond to the left-hand side, right-hand side,
/// and output of the constraint equation, respectively.
pub(crate) type Constraint<F> = (Vec<(usize, F)>, Vec<(usize, F)>, Vec<(usize, F)>);

/// Configuration for Circom circuit processing.
///
/// This structure holds the configuration necessary to handle R1CS files and witness calculation.
/// It includes the [`R1CS`] structure, a mutex-protected [`WitnessCalculator`], and a flag for sanity checks.
#[derive(Debug)]
pub struct CircomConfig<F: PrimeField> {
    pub r1cs: R1CS<F>,
    pub wtns: Mutex<WitnessCalculator>,
    pub sanity_check: bool,
}

impl<F: PrimeField> CircomConfig<F> {
    /// Create a new [`CircomConfig`] instance.
    ///
    /// `wtns`: Path to the WASM file used for witness calculation.
    /// `r1cs`: Path to the R1CS file representing the circuit constraints.
    ///
    /// Returns a result containing the new [`CircomConfig`] instance or an error if the files
    /// cannot be loaded or parsed correctly.
    pub fn new(wtns: impl AsRef<Path>, r1cs: impl AsRef<Path>) -> Result<Self> {
        let path_wtns_string = wtns.as_ref().to_str().ok_or(FilenameError)?.to_string();
        let path_r1cs_string = r1cs.as_ref().to_str().ok_or(FilenameError)?.to_string();

        let wtns = Mutex::new(WitnessCalculator::new(wtns).map_err(|err| {
            WitnessCalculatorInstantiationError {
                path: path_wtns_string,
                source: err.into(),
            }
        })?);
        let r1cs = load_r1cs(r1cs).map_err(|err| LoadR1CSError {
            path: path_r1cs_string,
            source: err.into(),
        })?;
        Ok(Self {
            wtns,
            r1cs,
            sanity_check: false,
        })
    }
}
