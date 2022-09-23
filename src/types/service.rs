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

//! Servicios

use std::fmt;
use std::str;

use serde::{Deserialize, Serialize};

use crate::error::EpbdError;

/// Uso al que está destinada la energía
///
/// Algunos servicios pueden estar incluidos ya en el consumo de otros, como podría ser el
/// caso del consumo para HU en CAL, de DHU en REF o VEN en CAL y/o REF.
///
/// También debe tenerse en cuenta que algunos servicios, como la iluminación pueden considerarse
/// no EPB en algunos casos (p.e. residencial privado) y en ese caso no deben indicarse los consumos
/// como ILU sino como NEPB
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Service {
    /// DHW
    ACS,
    /// Heating (including humidification)
    CAL,
    /// Cooling (including dehumidification)
    REF,
    /// Ventilation, including heat recovery (when separate from heating or cooling)
    VEN,
    /// Lighting (only when considered as EPB use)
    ILU,
    /// Generic non EPB use
    NEPB,
    /// Energy feeding an electricity cogeneration system
    /// It accounts for energy used for electricity generation and excludes all
    /// energy that can attributed to thermal use
    COGEN,
}

impl Service {
    /// List of all available services
    pub const SERVICES_ALL: [Service; 7] = [
        Service::ACS,
        Service::CAL,
        Service::REF,
        Service::VEN,
        Service::ILU,
        Service::NEPB,
        Service::COGEN,
    ];

    /// List EPB services
    pub const SERVICES_EPB: [Service; 5] = [
        Service::ACS,
        Service::CAL,
        Service::REF,
        Service::VEN,
        Service::ILU,
    ];

    /// Check if service is an EPB service
    /// This doesn't include the NEPB and GEN services
    pub fn is_epb(&self) -> bool {
        *self != Self::NEPB && *self != Self::COGEN
    }

    /// Check if service is a non EPB service
    /// This doesn't include the GEN service
    pub fn is_nepb(&self) -> bool {
        *self == Self::NEPB
    }

    /// Check if service is for electricity cogeneration
    pub fn is_cogen(&self) -> bool {
        *self == Self::COGEN
    }
}

impl str::FromStr for Service {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Service, Self::Err> {
        match s {
            "ACS" => Ok(Service::ACS),
            "CAL" => Ok(Service::CAL),
            "REF" => Ok(Service::REF),
            "VEN" => Ok(Service::VEN),
            "ILU" => Ok(Service::ILU),
            "NEPB" => Ok(Service::NEPB),
            "COGEN" => Ok(Service::COGEN),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
