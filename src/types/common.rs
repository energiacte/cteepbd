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

use crate::error::EpbdError;

// ==================== Common types (components + weighting factors)

// -------------------- Carrier

/// Vector energético (energy carrier).
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Carrier {
    /// Electricity
    ELECTRICIDAD,
    /// Environment thermal energy or from solar origin
    MEDIOAMBIENTE,
    /// Biofuel
    BIOCARBURANTE,
    /// Biomass
    BIOMASA,
    /// Densified biomass (pellets)
    BIOMASADENSIFICADA,
    /// Coal
    CARBON,
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
    /// No carrier, placeholder
    NDEF,
}

impl str::FromStr for Carrier {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Carrier, Self::Err> {
        match s {
            "ELECTRICIDAD" => Ok(Carrier::ELECTRICIDAD),
            "MEDIOAMBIENTE" => Ok(Carrier::MEDIOAMBIENTE),
            "BIOCARBURANTE" => Ok(Carrier::BIOCARBURANTE),
            "BIOMASA" => Ok(Carrier::BIOMASA),
            "BIOMASADENSIFICADA" => Ok(Carrier::BIOMASADENSIFICADA),
            "CARBON" => Ok(Carrier::CARBON),
            "GASNATURAL" => Ok(Carrier::GASNATURAL),
            "GASOLEO" => Ok(Carrier::GASOLEO),
            "GLP" => Ok(Carrier::GLP),
            "RED1" => Ok(Carrier::RED1),
            "RED2" => Ok(Carrier::RED2),
            "-" => Ok(Carrier::NDEF),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for Carrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NDEF => write!(f, "-"),
            _ => write!(f, "{:?}", self),
        }
    }
}

// ==================== Energy Components

// -------------------- CType

/// Tipo del componente (energía consumida o producida)
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CType {
    /// Produced energy by system Y (E_pr_el for electricity, Q_X_Y_out for ambient energy or solar systems)
    PRODUCCION,
    /// Used energy by system Y to provide service X (E_X_Y_in)
    CONSUMO,
}

impl str::FromStr for CType {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<CType, Self::Err> {
        match s {
            "PRODUCCION" => Ok(CType::PRODUCCION),
            "CONSUMO" => Ok(CType::CONSUMO),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// -------------------- CSubtype

/// Subtipo del componente (origen o destino de la energía)
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CSubtype {
    /// On site energy source
    INSITU,
    /// Cogeneration energy source
    COGENERACION,
    /// EPB use
    EPB,
    /// Non EPB use
    NEPB,
}

impl str::FromStr for CSubtype {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<CSubtype, Self::Err> {
        match s {
            "INSITU" => Ok(CSubtype::INSITU),
            "COGENERACION" => Ok(CSubtype::COGENERACION),
            "EPB" => Ok(CSubtype::EPB),
            "NEPB" => Ok(CSubtype::NEPB),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for CSubtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// -------------------- Service

/// Uso al que está destinada la energía
///
/// Algunos servicios pueden estar incluidos ya en el consumo de otros, como podría ser el
/// caso del consumo para HU en CAL, de DHU en REF o VEN en CAL y/o REF.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Service {
    /// DHW
    ACS,
    /// Heating
    CAL,
    /// Cooling
    REF,
    /// Ventilation
    VEN,
    /// Lighting
    ILU,
    /// Humidification
    HU,
    /// Dehumidification
    DHU,
    /// Building automation and control
    BAC,
    /// Undefined or generic use
    NDEF,
    // TODO: Electricity cogeneration (electrical use, excluded thermal use)
    // COGEN,
    // TODO: Non EPB uses
    // NEPB,
}

/// Lista de usos disponibles
pub const SERVICES: [Service; 9] = [
    Service::ACS,
    Service::CAL,
    Service::REF,
    Service::VEN,
    Service::ILU,
    Service::HU,
    Service::DHU,
    Service::BAC,
    Service::NDEF,
    //Service::COGEN,
    //Service::NEPB,
];

impl str::FromStr for Service {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Service, Self::Err> {
        match s {
            "ACS" => Ok(Service::ACS),
            "WATERSYSTEMS" => Ok(Service::ACS),
            "CAL" => Ok(Service::CAL),
            "HEATING" => Ok(Service::CAL),
            "REF" => Ok(Service::REF),
            "COOLING" => Ok(Service::REF),
            "VEN" => Ok(Service::VEN),
            "FANS" => Ok(Service::VEN),
            "ILU" => Ok(Service::ILU),
            "HU" => Ok(Service::HU),
            "DHU" => Ok(Service::DHU),
            "BAC" => Ok(Service::BAC),
            "NDEF" => Ok(Service::NDEF),
            // "COGEN" => Ok(Service::COGEN),
            "" => Ok(Service::default()),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for Service {
    fn default() -> Service {
        Service::NDEF
    }
}

// impl Service {
//     // Is this an EPB service? How does it interact with cogeneration?
//     fn is_epb(&self) -> bool {
//         self != Service::NEPB && self != Service::COGEN
//     }
//     fn is_nepb(&self) -> bool {
//         self == Service::NEPB
//     }
// }
