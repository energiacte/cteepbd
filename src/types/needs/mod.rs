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

use super::{CType, HasValues, Service};
use crate::error::EpbdError;
use crate::vecops::vecvecsum;

// -------------------- Building Energy Needs Component
// Define basic Building Energy Needs Component type and a container of all Building needs
// The component is used to express energy needs of the whole building provide service X (X=CAL/REF/ACS) (Q_X_nd_t)
// The component stores building needs for heating (CAL), cooling (REF) and domestic heat water (ACS)

/// Demandas del edificio
#[allow(non_snake_case)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BuildingNeeds {
    /// Timestep building energy needs to provide the domestic heat water service, Q_DHW_nd_t. kWh
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ACS: Option<Vec<f32>>,
    /// Timestep building energy needs to provide the heating service, Q_H_nd_t. kWh
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub CAL: Option<Vec<f32>>,
    /// Timestep building energy needs to provide the cooling service, Q_C_nd_t. kWh
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub REF: Option<Vec<f32>>,
}

impl BuildingNeeds {
    /// Añade elemento de demanda del edificio, sumando los valores si ya se han definido para ese servicio
    pub fn add(&mut self, need: Needs) -> Result<(), EpbdError> {
        let update = |cur_values: &Option<Vec<f32>>, new_values| {
            if let Some(nd) = cur_values {
                Some(vecvecsum(nd, new_values))
            } else {
                Some(new_values.to_owned())
            }
        };
        match need.service {
            Service::ACS => self.ACS = update(&self.ACS, &need.values),
            Service::CAL => self.CAL = update(&self.CAL, &need.values),
            Service::REF => self.REF = update(&self.REF, &need.values),
            _ => {
                return Err(EpbdError::WrongInput(format!(
                    "Demanda de edificio con servicio no contemplado por el programa: {}",
                    need.service
                )))
            }
        };
        Ok(())
    }
}

/// Componente de demanda de edificio.
///
/// Se serializa como: `DEMANDA, servicio, vals... # comentario`
///
/// - servicio == CAL / REF / ACS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Needs {
    /// End use (CAL, REF, ACS)
    pub service: Service,
    /// List of timestep energy needs for the building to provide service X, Q_X_nd_t. kWh
    pub values: Vec<f32>,
}

impl HasValues for Needs {
    fn values(&self) -> &[f32] {
        &self.values
    }
}

impl fmt::Display for Needs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_list = self
            .values
            .iter()
            .map(|v| format!("{:.2}", v))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "DEMANDA, {}, {}", self.service, value_list)
    }
}

impl str::FromStr for Needs {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Needs, Self::Err> {
        // Split comment from the rest of fields
        let items: Vec<&str> = s.trim().splitn(2, '#').map(str::trim).collect();
        let items: Vec<&str> = items[0].split(',').map(str::trim).collect();

        // Minimal possible length (DEMANDA + Service + 1 value)
        if items.len() < 3 {
            return Err(EpbdError::ParseError(s.into()));
        };

        // Check type
        match items[0].parse() {
            Ok(CType::DEMANDA) => {}
            _ => {
                return Err(EpbdError::ParseError(format!(
                    "No se reconoce el formato como elemento de Demanda: {}",
                    s
                )))
            }
        };

        // Check valid service field CAL, REF, ACS
        let service = items[1].parse()?;
        if ![Service::CAL, Service::REF, Service::ACS].contains(&service) {
            return Err(EpbdError::ParseError(format!(
                "Servicio no soportado en componente de DEMANDA del edificio: {}",
                service
            )));
        }

        // Collect energy values from the service field on
        let values = items[2..]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;

        Ok(Needs { service, values })
    }
}

// ========================== Tests

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn component_building_needs() {
        // zone energy needs component
        let component1 = Needs {
            service: "REF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, -6.0, -7.0, -8.0, -9.0, 10.0, 11.0, 12.0,
            ],
        };
        let component1str = "DEMANDA, REF, 1.00, 2.00, 3.00, 4.00, 5.00, -6.00, -7.00, -8.00, -9.00, 10.00, 11.00, 12.00";
        assert_eq!(component1.to_string(), component1str);

        // roundtrip building from/to string
        assert_eq!(
            component1str.parse::<Needs>().unwrap().to_string(),
            component1str
        );
    }
}
