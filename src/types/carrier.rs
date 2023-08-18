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

//! Vectores energéticos

use std::fmt;
use std::str;

use serde::{Deserialize, Serialize};

use super::ProdSource;

use crate::error::EpbdError;

/// Vector energético (energy carrier).
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Carrier {
    /// Environment thermal energy (from heat pumps and other)
    EAMBIENTE,
    /// Biofuel
    BIOCARBURANTE,
    /// Biomass
    BIOMASA,
    /// Densified biomass (pellets)
    BIOMASADENSIFICADA,
    /// Coal
    CARBON,
    /// Electricity
    ELECTRICIDAD,
    /// Natural gas
    GASNATURAL,
    /// Diesel oil
    GASOLEO,
    /// LPG - Liquefied petroleum gas
    GLP,
    /// Generic energy carrier 1
    RED1,
    /// Generic energy carrier 2
    RED2,
    /// Thermal energy from solar collectors
    TERMOSOLAR,
}

/// TODO: La clasificación de los vectores en función del perímetro debería hacerse
/// TODO: en la propia definición de esos vectores
impl Carrier {
    /// Vectores considerados dentro del perímetro NEARBY (a excepción de la ELECTRICIDAD in situ).
    pub const NRBY: [Carrier; 6] = [
        Carrier::BIOMASA,
        Carrier::BIOMASADENSIFICADA,
        Carrier::RED1,
        Carrier::RED2,
        Carrier::EAMBIENTE,
        Carrier::TERMOSOLAR,
    ]; // Ver B.23. Solo biomasa sólida

    /// Vectores considerados dentro del perímetro ONSITE (a excepción de la ELECTRICIDAD in situ).
    pub const ONST: [Carrier; 2] = [Carrier::EAMBIENTE, Carrier::TERMOSOLAR];

    /// Is this a carrier from the onsite or nearby perimeter?
    pub fn is_nearby(&self) -> bool {
        Carrier::NRBY.contains(self)
    }

    /// Is this a carrier from the onsite perimeter?
    pub fn is_onsite(&self) -> bool {
        Carrier::ONST.contains(self)
    }
}

impl str::FromStr for Carrier {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Carrier, Self::Err> {
        match s {
            "EAMBIENTE" => Ok(Carrier::EAMBIENTE),
            "BIOCARBURANTE" => Ok(Carrier::BIOCARBURANTE),
            "BIOMASA" => Ok(Carrier::BIOMASA),
            "BIOMASADENSIFICADA" => Ok(Carrier::BIOMASADENSIFICADA),
            "CARBON" => Ok(Carrier::CARBON),
            "ELECTRICIDAD" => Ok(Carrier::ELECTRICIDAD),
            "GASNATURAL" => Ok(Carrier::GASNATURAL),
            "GASOLEO" => Ok(Carrier::GASOLEO),
            "GLP" => Ok(Carrier::GLP),
            "RED1" => Ok(Carrier::RED1),
            "RED2" => Ok(Carrier::RED2),
            "TERMOSOLAR" => Ok(Carrier::TERMOSOLAR),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for Carrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::convert::From<ProdSource> for Carrier {
    fn from(value: ProdSource) -> Self {
        match value {
            ProdSource::EL_INSITU => Carrier::ELECTRICIDAD,
            ProdSource::EL_COGEN => Carrier::ELECTRICIDAD,
            ProdSource::TERMOSOLAR => Carrier::TERMOSOLAR,
            ProdSource::EAMBIENTE => Carrier::EAMBIENTE,
        }
    }
}