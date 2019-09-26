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
Energy components
=================

Define Components type (Compoment list + Metadata).

*/

use std::fmt;
use std::str;

use crate::types::base::{Component, Meta, MetaVec};
use crate::EpbdError;

/// List of component data bundled with its metadata
///
/// #META CTE_AREAREF: 100.5
/// ELECTRICIDAD,CONSUMO,EPB,16.39,13.11,8.20,7.38,4.10,4.92,6.56,5.74,4.10,6.56,9.84,13.11
/// ELECTRICIDAD,PRODUCCION,INSITU,8.20,6.56,4.10,3.69,2.05,2.46,3.28,2.87,2.05,3.28,4.92,6.56
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Component list
    pub cmeta: Vec<Meta>,
    /// Metadata
    pub cdata: Vec<Component>,
}

impl MetaVec for Components {
    fn get_metavec(&self) -> &Vec<Meta> {
        &self.cmeta
    }
    fn get_mut_metavec(&mut self) -> &mut Vec<Meta> {
        &mut self.cmeta
    }
}

impl fmt::Display for Components {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metalines = self
            .cmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self
            .cdata
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n{}", metalines, datalines)
    }
}

impl str::FromStr for Components {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Components, Self::Err> {
        let s_nobom = if s.starts_with("\u{feff}") {
            &s[3..]
        } else {
            s
        };
        let lines: Vec<&str> = s_nobom.lines().map(str::trim).collect();
        let metalines = lines
            .iter()
            .filter(|l| l.starts_with("#META") || l.starts_with("#CTE_"));
        let datalines = lines
            .iter()
            .filter(|l| !(l.starts_with('#') || l.starts_with("vector,") || l.is_empty()));
        let cmeta = metalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Meta>, _>>()?;
        let cdata = datalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Component>, _>>()?;
        {
            let cdata_lens: Vec<_> = cdata.iter().map(|e| e.values.len()).collect();
            if cdata_lens.iter().max().unwrap() != cdata_lens.iter().min().unwrap() {
                return Err(EpbdError::ComponentsParseError(s.into()));
            }
        }
        Ok(Components { cmeta, cdata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcomponents() {
        let tcomponents1 = "#META CTE_AREAREF: 100.5
ELECTRICIDAD, CONSUMO, EPB, NDEF, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56";

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", tcomponents1.parse::<Components>().unwrap()),
            tcomponents1
        );
    }
}