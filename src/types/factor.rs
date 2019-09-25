// Copyright (c) 2018 Ministerio de Fomento
//                    Instituto de Ciencias de la Construcción Eduardo Torroja (IETcc-CSIC)

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

// Author(s): Rafael Villar Burke <pachi@ietcc.csic.es>

use std::fmt;
use std::str;

use crate::types::{basic::*, meta::*, rennrenco2::*};
use crate::{EpbdError};

/// Define Factor and Factors (Factor list + Metadata) types

/// Weighting Factor Struct
///
/// It can represent the renewable and non renewable primary energy weighting factors,
/// but can be used for CO2 or any other indicators depending on how the values are obtained.
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
        ren: f32,
        nren: f32,
        co2: f32,
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

    /// Get factors as RenNrenCo2 struct
    pub fn factors(&self) -> RenNrenCo2 {
        RenNrenCo2 {
            ren: self.ren,
            nren: self.nren,
            co2: self.co2,
        }
    }
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let comment = if self.comment != "" {
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
            return Err(EpbdError::WFactorParseError(s.into()));
        };
        let carrier: Carrier = items[0]
            .parse()
            .map_err(|_| EpbdError::CarrierUnknown(items[0].into()))?;
        let source: Source = items[1]
            .parse()
            .map_err(|_| EpbdError::SourceUnknown(items[1].into()))?;
        let dest: Dest = items[2]
            .parse()
            .map_err(|_| EpbdError::DestUnknown(items[2].into()))?;
        let step: Step = items[3]
            .parse()
            .map_err(|_| EpbdError::StepUnknown(items[3].into()))?;
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

// --------------------------- Factors --------------------------

/// List of weighting factors bundled with its metadata
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Factors {
    /// Weighting factors list
    pub wmeta: Vec<Meta>,
    /// Metadata
    pub wdata: Vec<Factor>,
}

impl Factors {
    // Remove nEPB weighting factors
    pub fn strip_nepb(&mut self) {
        self.wdata.retain(|e| e.dest != Dest::A_NEPB);
    }
}

impl MetaVec for Factors {
    fn get_metavec(&self) -> &Vec<Meta> {
        &self.wmeta
    }
    fn get_mut_metavec(&mut self) -> &mut Vec<Meta> {
        &mut self.wmeta
    }
}

impl fmt::Display for Factors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metalines = self
            .wmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self
            .wdata
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n{}", metalines, datalines)
    }
}

impl str::FromStr for Factors {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Factors, Self::Err> {
        let lines: Vec<&str> = s.lines().map(str::trim).collect();
        let metalines = lines
            .iter()
            .filter(|l| l.starts_with("#META") || l.starts_with("#CTE_"));
        let datalines = lines
            .iter()
            .filter(|l| !(l.starts_with('#') || l.starts_with("vector,") || l.is_empty()));
        let wmeta = metalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Meta>, _>>()?;
        let wdata = datalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Factor>, _>>()?;
        Ok(Factors { wmeta, wdata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Component;

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
        let factor2str = "ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";

        // consumer component
        assert_eq!(format!("{}", factor1), factor1str);

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", factor2str.parse::<Component>().unwrap()),
            factor2str
        );
    }
}