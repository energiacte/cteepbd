// Copyright (c) 2018-2019  Ministerio de Fomento
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

/*!
Weighting factors
=================

Define Factors typr (Factor list + Metadata).

*/

use std::fmt;
use std::str;

use crate::{
    error::EpbdError,
    types::{Dest, Factor, Meta, MetaVec},
};

// --------------------------- Factors

/// List of weighting factors bundled with its metadata
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Factors {
    /// Weighting factors list
    pub wmeta: Vec<Meta>,
    /// Metadata
    pub wdata: Vec<Factor>,
}

impl Factors {
    /// Remove non EPB weighting factors from the factor list
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

    #[test]
    fn tfactors() {
        let tfactors1 = "#META CTE_FUENTE: RITE2014
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954, 0.331 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para producir electricidad in situ";

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", tfactors1.parse::<Factors>().unwrap()),
            tfactors1
        );
    }
}
