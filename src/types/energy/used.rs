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

use crate::types::{Carrier, HasValues, Service};
use crate::error::EpbdError;

// -------------------- EUsed Energy Component
// Define basic EUsed Energy Component type

/// Componente de energía usada (consumos). E_X;gen,i;in;cr,j;t
///
/// Representa el consumo de energía del vector energético j
/// para el servicio X en el generador i, para los distintos pasos de cálculo t,
///
/// Las cantidades de energía de combustibles son en relación al poder calorífico superior.
/// Subsistema: generacion + almacenamiento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EUsed {
    /// System or part id (generator i)
    /// This can identify the system linked to this energy use.
    /// By default, id=0 means the whole building systems.
    /// Negative numbers should represent ficticious systems (such as the reference ones)
    /// A value greater than 0 identifies a specific system that is using some energy
    pub id: i32,
    /// Carrier name
    pub carrier: Carrier,
    /// End use
    pub service: Service,
    /// List of timestep energy use for the current carrier and service. kWh
    pub values: Vec<f32>,
    /// Descriptive comment string
    /// This can also be used to label a component as auxiliary energy use
    /// by including in this field the "CTEEPBD_AUX" tag
    pub comment: String,
}

impl HasValues for EUsed {
    fn values(&self) -> &[f32] {
        &self.values
    }
}

impl fmt::Display for EUsed {
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
            "{}, CONSUMO, {}, {}, {}{}",
            self.id, self.service, self.carrier, valuelist, comment
        )
    }
}

impl str::FromStr for EUsed {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<EUsed, Self::Err> {
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
            Err(_) => (0, 0_i32),
        };

        // Check type
        let ctype = items[baseidx];
        if ctype != "CONSUMO" {
            return Err(EpbdError::ParseError(format!(
                "Componente de energía consumida con formato incorrecto: {}",
                s
            )));
        }

        // Check service field. May be missing in legacy versions
        let service = items[baseidx + 1].parse()?;

        let carrier: Carrier = items[baseidx + 2].parse()?;

        // Collect energy values from the service field on
        let values: Vec<_> = items[baseidx + 3..]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<_, _>>()?;

        Ok(EUsed {
            id,
            carrier,
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
    fn components_used_energy() {
        // EUsed energy component
        let component1 = EUsed {
            id: 0,
            carrier: "ELECTRICIDAD".parse().unwrap(),
            service: "NDEF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario cons 1".into(),
        };
        let component1str = "0, CONSUMO, NDEF, ELECTRICIDAD, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario cons 1";
        let component1strlegacy = "CONSUMO, NDEF, ELECTRICIDAD, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario cons 1";
        assert_eq!(component1.to_string(), component1str);

        // roundtrip building from/to string
        assert_eq!(
            component1str.parse::<EUsed>().unwrap().to_string(),
            component1str
        );

        // roundtrip building from/to legacy string
        assert_eq!(
            component1strlegacy.parse::<EUsed>().unwrap().to_string(),
            component1str
        );
    }
}
