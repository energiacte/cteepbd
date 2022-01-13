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

use std::convert::TryFrom;
use std::fmt;
use std::str;

use serde::{Deserialize, Serialize};

use crate::{error::EpbdError, types::RenNrenCo2};

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

// -------------------- CType

/// Tipo del componente (energía consumida o producida)
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CType {
    /// Produced energy
    PRODUCCION,
    /// Consumed energy
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
    // TODO: Electricity cogeneration (share of electrical use, excluding thermal use)
    // COGEN,
    // TODO: Auxiliary energy
    // Should this be a service or a distinct component
    // AUX,
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
    //Service::AUX,
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
            // "AUX" => Ok(Service::AUX),
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

// -------------------- Component
// Define basic Component and Components (Compoment list + Metadata) types

/// Componente de energía.
///
/// Representa la producción o consumo de energía para cada paso de cálculo
/// y a lo largo del periodo de cálculo, para cada tipo, subtipo y uso de la energía.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// System or part id
    /// This can identify the system or part linked to this component.
    /// By default, id=0 means the whole building
    /// A value that is not 0 could identify the system that generates or uses some energy
    pub id: u8,
    /// Carrier name
    pub carrier: Carrier,
    /// Component type
    /// - `PRODUCCION` for produced / generated energy components
    /// - `CONSUMO` for consumed / used energy components
    pub ctype: CType,
    /// Energy origin or end use type
    /// - `INSITU` or `COGENERACION` for generated energy component types
    /// - `EPB` or `NEPB` for used energy component types
    pub csubtype: CSubtype,
    /// End use
    pub service: Service,
    /// List of energy values, one value for each timestep
    pub values: Vec<f32>,
    /// Descriptive comment string
    pub comment: String,
}

impl Component {
    /// Sum of values for the component
    pub fn values_sum(&self) -> f32 {
        self.values.iter().sum::<f32>()
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let valuelist = self
            .values
            .iter()
            .map(|v| format!("{:.2}", v))
            .collect::<Vec<_>>()
            .join(", ");
        let comment = if !self.comment.is_empty() {
            format!(" # {}", self.comment)
        } else {
            "".to_owned()
        };
        write!(
            f,
            "{}, {}, {}, {}, {}, {}{}",
            self.id, self.carrier, self.ctype, self.csubtype, self.service, valuelist, comment
        )
    }
}

impl str::FromStr for Component {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Component, Self::Err> {
        use self::CSubtype::*;
        use self::CType::*;
        use self::Carrier::{ELECTRICIDAD, MEDIOAMBIENTE};

        // Split comment from the rest of fields
        let items: Vec<&str> = s.trim().splitn(2, '#').map(str::trim).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(str::trim).collect();

        // Minimal possible length (carrier + type + subtype + 1 value)
        if items.len() < 4 {
            return Err(EpbdError::ParseError(s.into()));
        };

        let (baseidx, id) = match items[0].parse() {
            Ok(id) => (1, id),
            Err(_) => (0, 0_u8),
        };

        let carrier: Carrier = items[baseidx].parse()?;
        let ctype: CType = items[baseidx + 1].parse()?;
        let csubtype: CSubtype = items[baseidx + 2].parse()?;

        // Check coherence of ctype and csubtype
        let subtype_belongs_to_type = match ctype {
            CONSUMO => matches!(csubtype, EPB | NEPB),
            PRODUCCION => match csubtype {
                INSITU => carrier == ELECTRICIDAD || carrier == MEDIOAMBIENTE,
                COGENERACION => carrier == ELECTRICIDAD,
                _ => false,
            },
        };
        if !subtype_belongs_to_type {
            return Err(EpbdError::ParseError(s.into()));
        }

        // Check service field. May be missing in legacy versions
        let (valuesidx, service) = match items[baseidx + 3].parse() {
            Ok(s) => (baseidx + 4, s),
            Err(_) => (baseidx + 3, Service::default()),
        };

        // Collect energy values from the service field on
        let values = items[valuesidx..]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;

        Ok(Component {
            id,
            carrier,
            ctype,
            csubtype,
            service,
            values,
            comment,
        })
    }
}

// ==================== Weighting factors

// -------------------- Source

/// Fuente de origen de la energía
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Source {
    /// Grid source
    RED,
    /// Insitu generation source
    INSITU,
    /// Cogeneration source
    COGENERACION,
}

impl str::FromStr for Source {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Source, Self::Err> {
        match s {
            "RED" => Ok(Source::RED),
            "INSITU" => Ok(Source::INSITU),
            "COGENERACION" => Ok(Source::COGENERACION),
            _ => Err(EpbdError::ParseError(s.into())),
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<CSubtype> for Source {
    type Error = EpbdError;
    fn try_from(subtype: CSubtype) -> Result<Self, Self::Error> {
        match subtype {
            CSubtype::INSITU => Ok(Self::INSITU),
            CSubtype::COGENERACION => Ok(Self::COGENERACION),
            _ => Err(EpbdError::ParseError(format!(
                "CSubtype as Source {}",
                subtype
            ))),
        }
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
            // Legacy
            "to_grid" => Ok(Dest::A_RED),
            "to_nEPB" => Ok(Dest::A_NEPB),
            "input" => Ok(Dest::SUMINISTRO),
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

// ------------------ Weighting Factor

/// Factor de paso
///
/// Representa la fracción renovable, no renovable y emisiones de una unidad de energía final,
/// evaluados en el paso de cálculo y para un vector y una fuente determinados.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factor {
    /// Energy carrier
    pub carrier: Carrier,
    /// Carrier source (`RED`, `INSITU` or `COGENERACION`)
    pub source: Source,
    /// Destination use of the energy (`SUMINISTRO`, `A_RED`, `A_NEPB`)
    pub dest: Dest,
    /// Evaluation step
    pub step: Step,
    /// Renewable primary energy for each end use unit of this carrier
    pub ren: f32,
    /// Non renewable primary energy for each end use unit of this carrier
    pub nren: f32,
    /// CO2 emissions for each end use unit of this carrier
    pub co2: f32,
    /// Descriptive comment string for the weighting factor
    pub comment: String,
}

impl Factor {
    /// Constructor
    pub fn new<T: Into<String>>(
        carrier: Carrier,
        source: Source,
        dest: Dest,
        step: Step,
        RenNrenCo2 { ren, nren, co2 }: RenNrenCo2,
        comment: T,
    ) -> Self {
        Self {
            carrier,
            source,
            dest,
            step,
            ren,
            nren,
            co2,
            comment: comment.into(),
        }
    }

    /// Obtener los factores de paso como estructura RenNrenCo2
    pub fn factors(&self) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren,
            nren: self.nren,
            co2: self.co2,
        }
    }

    /// Copia los factores desde una estructura RenNRenCo2
    pub fn set_values(&mut self, &values: &RenNrenCo2) {
        self.ren = values.ren;
        self.nren = values.nren;
        self.co2 = values.co2;
    }
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let comment = if !self.comment.is_empty() {
            format!(" # {}", self.comment)
        } else {
            "".to_owned()
        };
        write!(
            f,
            "{}, {}, {}, {}, {:.3}, {:.3}, {:.3}{}",
            self.carrier, self.source, self.dest, self.step, self.ren, self.nren, self.co2, comment
        )
    }
}

impl str::FromStr for Factor {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Factor, Self::Err> {
        let items: Vec<&str> = s.trim().splitn(2, '#').map(str::trim).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(str::trim).collect();
        if items.len() < 7 {
            return Err(EpbdError::ParseError(s.into()));
        };
        let carrier: Carrier = items[0]
            .parse()
            .map_err(|_| EpbdError::ParseError(items[0].into()))?;
        let source: Source = items[1]
            .parse()
            .map_err(|_| EpbdError::ParseError(items[1].into()))?;
        let dest: Dest = items[2]
            .parse()
            .map_err(|_| EpbdError::ParseError(items[2].into()))?;
        let step: Step = items[3]
            .parse()
            .map_err(|_| EpbdError::ParseError(items[3].into()))?;
        let ren: f32 = items[4].parse()?;
        let nren: f32 = items[5].parse()?;
        let co2: f32 = items[6].parse()?;
        Ok(Factor {
            carrier,
            source,
            dest,
            step,
            ren,
            nren,
            co2,
            comment,
        })
    }
}

// ========================== Tests

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tcomponent() {
        let component1 = Component {
            id: 0,
            carrier: "ELECTRICIDAD".parse().unwrap(),
            ctype: "CONSUMO".parse().unwrap(),
            csubtype: "EPB".parse().unwrap(),
            service: "REF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario cons 1".into(),
        };
        let component1str = "0, ELECTRICIDAD, CONSUMO, EPB, REF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario cons 1";
        let component2 = Component {
            id: 0,
            carrier: "ELECTRICIDAD".parse().unwrap(),
            ctype: "PRODUCCION".parse().unwrap(),
            csubtype: "INSITU".parse().unwrap(),
            service: "NDEF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario prod 1".into(),
        };
        let component2str = "0, ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";
        let component2strlegacy = "ELECTRICIDAD, PRODUCCION, INSITU, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";

        // consumer component
        assert_eq!(component1.to_string(), component1str);

        // producer component
        assert_eq!(component2.to_string(), component2str);

        // roundtrip building from/to string
        assert_eq!(
            component2str.parse::<Component>().unwrap().to_string(),
            component2str
        );
        // roundtrip building from/to string for legacy format
        assert_eq!(
            component2strlegacy
                .parse::<Component>()
                .unwrap()
                .to_string(),
            component2str
        );
    }

    #[test]
    fn tfactor() {
        let factor1 = Factor {
            carrier: "ELECTRICIDAD".parse().unwrap(),
            source: "RED".parse().unwrap(),
            dest: "SUMINISTRO".parse().unwrap(),
            step: "A".parse().unwrap(),
            ren: 0.414,
            nren: 1.954,
            co2: 0.331,
            comment: "Electricidad de red paso A".into(),
        };
        let factor1str =
            "ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954, 0.331 # Electricidad de red paso A";
        let factor2str = "0, ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";

        // consumer component
        assert_eq!(factor1.to_string(), factor1str);

        // roundtrip building from/to string
        assert_eq!(
            factor2str.parse::<Component>().unwrap().to_string(),
            factor2str
        );
    }
}
