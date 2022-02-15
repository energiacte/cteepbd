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

    /// Is this a carrier from the nearby perimeter?
    pub fn is_nearby(&self) -> bool {
        Carrier::NRBY.contains(self)
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

// ==================== Energy Components

// -------------------- ProdSource

/// Fuente de origen de la energía producida
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProdSource {
    /// On site generated electricity
    EL_INSITU,
    /// On site cogenerated electricity
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

// -------------------- Service

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
    /// Heating
    CAL,
    /// Cooling
    REF,
    /// Ventilation, including heat recovery
    VEN,
    /// Lighting (only when considered as EPB use)
    ILU,
    /// Humidification, when not included in Heating
    HU,
    /// Dehumidification, when not included in Cooling
    DHU,
    /// Building automation and control
    BAC,
    /// Generic non EPB use
    NEPB,
    /// Energy feeding an electricity cogeneration system
    /// It accounts for energy used for electricity generation and excludes all energy that can attributed to thermal use
    COGEN,
}

impl Service {
    /// List of all available services
    pub const SERVICES_ALL: [Service; 10] = [
        Service::ACS,
        Service::CAL,
        Service::REF,
        Service::VEN,
        Service::ILU,
        Service::HU,
        Service::DHU,
        Service::BAC,
        Service::NEPB,
        Service::COGEN,
    ];

    /// List EPB services
    pub const SERVICES_EPB: [Service; 8] = [
        Service::ACS,
        Service::CAL,
        Service::REF,
        Service::VEN,
        Service::ILU,
        Service::HU,
        Service::DHU,
        Service::BAC,
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
            "FANS" => Ok(Service::VEN),
            "ILU" => Ok(Service::ILU),
            "HU" => Ok(Service::HU),
            "DHU" => Ok(Service::DHU),
            "BAC" => Ok(Service::BAC),
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

// -------------------- Dest

/// Destino de la energía
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
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
