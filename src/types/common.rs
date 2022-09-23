// Copyright (c) 2018-2022  Ministerio de Fomento
//                          Instituto de Ciencias de la Construcción Eduardo Torroja (IETcc-CSIC)

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// Author(s): Rafael Villar Burke <pachi@ietcc.csic.es>,
//            Daniel Jiménez González <dani@ietcc.csic.es>,
//            Marta Sorribes Gil <msorribes@ietcc.csic.es>

use std::fmt;
use std::str;

use serde::{Deserialize, Serialize};

use super::ProdSource;

use crate::error::EpbdError;

// ================= Weighting Factors =============

// -------------------- Source

/// Fuente de origen de la energía
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    /// Grid source
    RED,
    /// On site generation source
    INSITU,
    /// Cogeneration source
    COGEN,
}

impl str::FromStr for Source {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Source, Self::Err> {
        match s {
            "RED" => Ok(Source::RED),
            "INSITU" => Ok(Source::INSITU),
            "COGEN" => Ok(Source::COGEN),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::convert::From<ProdSource> for Source {
    fn from(value: ProdSource) -> Self {
        match value {
            ProdSource::EL_INSITU => Source::INSITU,
            ProdSource::EL_COGEN => Source::COGEN,
            ProdSource::TERMOSOLAR => Source::INSITU,
            ProdSource::EAMBIENTE => Source::INSITU,
        }
    }
}

// -------------------- Dest

/// Destino de la energía
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dest {
    /// Building delivery destination
    SUMINISTRO,
    /// Grid destination
    A_RED,
    /// Non EPB uses destination
    A_NEPB,
}

impl str::FromStr for Dest {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Dest, Self::Err> {
        match s {
            "SUMINISTRO" => Ok(Dest::SUMINISTRO),
            "A_RED" => Ok(Dest::A_RED),
            "A_NEPB" => Ok(Dest::A_NEPB),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for Dest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// -------------------- Step

/// Paso de cálculo para el que se define el factor de paso
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    /// Calculation step A
    A,
    /// Calculation step B
    B,
}

impl str::FromStr for Step {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Step, Self::Err> {
        match s {
            "A" => Ok(Step::A),
            "B" => Ok(Step::B),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ================== Component Traits =====================

/// Elements that have a list of numeric values
pub trait HasValues {
    /// Get list of values
    fn values(&self) -> &[f32];

    /// Sum of all values
    fn values_sum(&self) -> f32 {
        self.values().iter().sum::<f32>()
    }

    /// Number of steps
    fn num_steps(&self) -> usize {
        self.values().len()
    }
}
