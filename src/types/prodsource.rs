// Copyright (c) 2018-2023  Ministerio de Fomento
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

//! Origen de la energía producida

use std::fmt;
use std::str;

use serde::{Deserialize, Serialize};

use super::Carrier;

use crate::error::EpbdError;

/// Fuente de origen de la energía producida
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProdSource {
    /// On site generated electricity
    EL_INSITU,
    /// On site co-generated electricity
    EL_COGEN,
    /// On site solar thermal
    TERMOSOLAR,
    /// On site ambient heat
    EAMBIENTE,
}

impl ProdSource {
    /// Priorities for electrical production sources
    pub fn get_priorities(carrier: Carrier) -> (bool, Vec<Self>) {
        match carrier {
            Carrier::ELECTRICIDAD => (true, vec![Self::EL_INSITU, Self::EL_COGEN]),
            _ => (false, vec![]),
        }
    }
}

impl str::FromStr for ProdSource {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<ProdSource, Self::Err> {
        match s {
            "EL_INSITU" => Ok(ProdSource::EL_INSITU),
            "EL_COGEN" => Ok(ProdSource::EL_COGEN),
            "TERMOSOLAR" => Ok(ProdSource::TERMOSOLAR),
            "EAMBIENTE" => Ok(ProdSource::EAMBIENTE),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for ProdSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
