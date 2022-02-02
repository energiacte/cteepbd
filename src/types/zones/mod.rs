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

use super::{HasValues, Service};
use crate::error::EpbdError;

// -------------------- Zone Energy Needs Component
// Define basic Zone Energy Needs Component type
// This component is used to express energy needs of this zone to provide service X (for zone i with i=0 for the whole building) (Q_X_nd_i)

/// Componente de zona.
///
/// Componente de datos de zonas del edificio
///
/// Se serializa como: `id, ZONA, DEMANDA, servicio, vals... # comentario`
///
/// - ZONA, DEMANDA, CAL / REF, meses
/// TODO: otros datos de ZONA (Ver EN 52000-1, 12.1 informe)
///
/// - ZONA, TEMPERATURA, EXT / INT, meses
/// - ZONA, RADIACION, HOR, meses
/// - ZONA, TRANSFERENCIA, TRANSMISION, meses
/// - ZONA, TRANSFERENCIA, VENTILACION, meses
/// - ZONA, GANANCIAS, SOLARES, meses
/// - ZONA, GANANCIAS, INTERNAS, meses
/// - Number of hours where temperature shedule limits are not met (CAL, REF)
/// - ZONA, HORASFC, TOT / CAL / REF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneNeeds {
    /// Zone id
    /// This identifies the part of the building linked to this component.
    /// By default, id=0 means the whole building (all zones)
    /// Negative numbers should represent ficticious elements (ficticious zones, such as reference ones)
    pub id: i32,
    /// End use
    pub service: Service,
    /// List of timestep energy needs for zone i (i=0 for the whole building) to provide service X, Q_X_nd_i_t. kWh
    /// Negative values means needs heating and positive values, needs cooling. kWh
    pub values: Vec<f32>,
    /// Descriptive comment string
    pub comment: String,
}

impl HasValues for ZoneNeeds {
    fn values(&self) -> &[f32] {
        &self.values
    }
}

impl fmt::Display for ZoneNeeds {
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
            "{}, ZONA, DEMANDA, {}, {}{}",
            self.id, self.service, valuelist, comment
        )
    }
}

impl str::FromStr for ZoneNeeds {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<ZoneNeeds, Self::Err> {
        // Split comment from the rest of fields
        let items: Vec<&str> = s.trim().splitn(2, '#').map(str::trim).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(str::trim).collect();

        // Minimal possible length (id + ZONA + DEMANDA + 1 value)
        if items.len() < 4 {
            return Err(EpbdError::ParseError(s.into()));
        };

        // Check ZONA and DEMANDA marker fields;
        if items[1] != "ZONA" || items[2] != "DEMANDA" {
            return Err(EpbdError::ParseError(format!(
                "No se reconoce el formato como elemento de Demanda: {}",
                s
            )));
        }

        // Zone Id
        let id = match items[0].parse() {
            Ok(id) => id,
            Err(_) => {
                return Err(EpbdError::ParseError(format!(
                    "Id erróneo en elemento de Demanda: {}",
                    s
                )))
            }
        };

        // Check service field
        let service = items[3].parse()?;

        // Collect energy values from the service field on
        let values = items[4..]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;

        Ok(ZoneNeeds {
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
    fn component_zone_needs() {
        // zone energy needs component
        let component1 = ZoneNeeds {
            id: 0,
            service: "REF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, -6.0, -7.0, -8.0, -9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario demanda zona 1".into(),
        };
        let component1str = "0, ZONA, DEMANDA, REF, 1.00, 2.00, 3.00, 4.00, 5.00, -6.00, -7.00, -8.00, -9.00, 10.00, 11.00, 12.00 # Comentario demanda zona 1";
        assert_eq!(component1.to_string(), component1str);

        // roundtrip building from/to string
        assert_eq!(
            component1str.parse::<ZoneNeeds>().unwrap().to_string(),
            component1str
        );
    }
}
