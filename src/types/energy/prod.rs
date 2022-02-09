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

use serde::{Deserialize, Serialize};

use crate::error::EpbdError;
use crate::types::{HasValues, ProdSource};

// -------------------- Produced Energy Component
// Define basic Produced Energy Component type

/// Componente de energía generada (producción). E_pr,i;cr,j;t
///
/// Representa la producción de energía con el vector energético j del sistema i
/// para cada paso de cálculo t, a lo largo del periodo de cálculo.
/// Subsistema: generacion + almacenamiento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EProd {
    /// System or part id
    /// This can identify the system linked to this component.
    /// By default, id=0 means a system attending the whole building
    /// Negative numbers should represent ficticious elements (such as reference systems)
    /// A value greater than 0 identies a specific energy generation system
    pub id: i32,
    /// Energy source
    /// - `ELINSITU | EL_COGEN | TERMOSOLAR | EAMBIENTE` for generated energy component types
    pub source: ProdSource,
    /// List of produced energy values, one value for each time step. kWh
    pub values: Vec<f32>,
    /// Descriptive comment string
    pub comment: String,
}

impl HasValues for EProd {
    fn values(&self) -> &[f32] {
        &self.values
    }
}

impl std::fmt::Display for EProd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            "{}, PRODUCCION, {}, {}{}",
            self.id, self.source, valuelist, comment
        )
    }
}

impl std::str::FromStr for EProd {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<EProd, Self::Err> {
        // Split comment from the rest of fields
        let items: Vec<&str> = s.trim().splitn(2, '#').map(str::trim).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(str::trim).collect();

        // Minimal possible length (type + source + 1 value)
        if items.len() < 3 {
            return Err(EpbdError::ParseError(s.into()));
        };

        let (baseidx, id) = match items[0].parse() {
            Ok(id) => (1, id),
            Err(_) => (0, 0_i32),
        };

        let ctype = items[baseidx];
        if ctype != "PRODUCCION" {
            return Err(EpbdError::ParseError(format!(
                "Componente de energía generada con formato incorrecto: {}",
                s
            )));
        }

        let source = items[baseidx + 1].parse()?;

        // Collect energy values from the service field on
        let values = items[baseidx + 2..]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()
            .map_err(|_| {
                EpbdError::ParseError(format!("se esperaban valores numéricos en línea `{}`", s))
            })?;

        Ok(EProd {
            id,
            source,
            values,
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
    fn produced_energy_component() {
        // produced energy component
        let component2 = EProd {
            id: 0,
            source: "EL_INSITU".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario prod 1".into(),
        };
        let component2str = "0, PRODUCCION, EL_INSITU, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";
        let component2strlegacy = "PRODUCCION, EL_INSITU, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";
        assert_eq!(component2.to_string(), component2str);

        // roundtrip building from/to string
        assert_eq!(
            component2str.parse::<EProd>().unwrap().to_string(),
            component2str
        );
        // roundtrip building from/to string for legacy format
        assert_eq!(
            component2strlegacy.parse::<EProd>().unwrap().to_string(),
            component2str
        );
    }
}
