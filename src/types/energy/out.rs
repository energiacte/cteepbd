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
use crate::types::{HasValues, Service};

// -------------------- System Energy Output Component
// Define basic System Energy Needs Component type
// This component is used to express energy output of this system to provide service X (for zone i with i=0 for the whole building) (E_X_gen_i_out_t)

/// Componente de generación.
///
/// Energía entregada o absorbida por los sistemas pertenecientes al subsistema de generación del edificio, E_X_gen_i_out
///
/// Se serializa como: `id, GEN, CARGA, servicio, vals... # comentario`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EOut {
    /// System id
    ///
    /// This identifies the system linked to this component.
    /// By default, id=0 means an hypothetical whole building system (encompassing all services)
    /// Negative numbers should represent ficticious elements (ficticious systems, such as the reference ones)
    pub id: i32,
    /// End use
    pub service: Service,
    /// Timestep energy output or absorbed energy values by system i to provide service X, E_X_gen_i_out_t. kWh
    /// Negative values means absorbed energy (e.g. by a chiller) and positive values means delivered energy (e.g. heat from a boiler) by the system. kWh
    pub values: Vec<f32>,
    /// Descriptive comment string
    pub comment: String,
}

impl HasValues for EOut {
    fn values(&self) -> &[f32] {
        &self.values
    }
}

impl fmt::Display for EOut {
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
            "{}, SALIDA, {}, {}{}",
            self.id, self.service, valuelist, comment
        )
    }
}

impl str::FromStr for EOut {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<EOut, Self::Err> {
        // Split comment from the rest of fields
        let items: Vec<&str> = s.trim().splitn(2, '#').map(str::trim).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(str::trim).collect();

        // Minimal possible length (id + GEN + SALIDA + 1 value)
        if items.len() < 4 {
            return Err(EpbdError::ParseError(s.into()));
        };

        // Check GEN and CARGA marker fields;
        if items[1] != "SALIDA" {
            return Err(EpbdError::ParseError(format!(
                "No se reconoce el formato como elemento de Salida del sistema: {}",
                s
            )));
        }

        // Zone Id
        let id = match items[0].parse() {
            Ok(id) => id,
            Err(_) => {
                return Err(EpbdError::ParseError(format!(
                    "Id erróneo en elemento de Salida del sistema: {}",
                    s
                )))
            }
        };

        // Check service field
        let service = items[2].parse()?;

        // Collect energy values from the service field on
        let values = items[3..]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;

        Ok(EOut {
            id,
            service,
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
    fn component_system_needs() {
        // zone energy needs component
        let component1 = EOut {
            id: 0,
            service: "REF".parse().unwrap(),
            values: vec![
                -1.0, -2.0, -3.0, -4.0, -5.0, -6.0, -7.0, -8.0, -9.0, -10.0, -11.0, -12.0,
            ],
            comment: "Comentario carga sobre sistema 0".into(),
        };
        let component1str = "0, SALIDA, REF, -1.00, -2.00, -3.00, -4.00, -5.00, -6.00, -7.00, -8.00, -9.00, -10.00, -11.00, -12.00 # Comentario carga sobre sistema 0";
        assert_eq!(component1.to_string(), component1str);

        // roundtrip building from/to string
        assert_eq!(
            component1str.parse::<EOut>().unwrap().to_string(),
            component1str
        );
    }
}
