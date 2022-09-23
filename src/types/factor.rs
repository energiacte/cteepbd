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

use super::{Carrier, ProdSource};

use crate::{error::EpbdError, types::RenNrenCo2};

// ==================== Weighting factors

// ------------------ Weighting Factor

/// Factor de paso
///
/// Representa la fracción renovable, no renovable y emisiones de una unidad de energía final,
/// evaluados en el paso de cálculo y para un vector y una fuente determinados.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factor {
    /// Energy carrier
    pub carrier: Carrier,
    /// Carrier source (`RED`, `INSITU` or `COGEN`)
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
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
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

// ========================== Tests

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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

        // consumer component
        assert_eq!(factor1.to_string(), factor1str);

        // roundtrip building from/to string
        assert_eq!(
            factor1str.parse::<Factor>().unwrap().to_string(),
            factor1str
        );
    }
}
