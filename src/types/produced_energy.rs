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

use super::{ProdOrigin, Carrier, HasValues, Service};
use crate::error::EpbdError;

// -------------------- Produced Energy Component
// Define basic Produced Energy Component type

/// Componente de energía generada.
///
/// Representa la producción de energía para cada paso de cálculo,
/// a lo largo del periodo de cálculo, para cada tipo de producción de energía.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducedEnergy {
    /// System or part id
    /// This can identify the system linked to this component.
    /// By default, id=0 means a system attending the whole building
    /// Negative numbers should represent ficticious elements (such as reference systems)
    /// A value greater than 0 identies a specific energy generation system
    pub id: i32,
    /// Carrier name
    pub carrier: Carrier,
    /// Energy origin
    /// - `INSITU` or `COGENERACION` for generated energy component types
    pub csubtype: ProdOrigin,
    /// End use
    pub service: Service,
    /// List of produced energy values, one value for each timestep. kWh
    pub values: Vec<f32>,
    /// Descriptive comment string
    pub comment: String,
}

impl HasValues for ProducedEnergy {
    fn values(&self) -> &[f32] {
        &self.values
    }
}

impl fmt::Display for ProducedEnergy {
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
            "{}, {}, PRODUCCION, {}, {}, {}{}",
            self.id, self.carrier, self.csubtype, self.service, valuelist, comment
        )
    }
}

impl str::FromStr for ProducedEnergy {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<ProducedEnergy, Self::Err> {
        use self::ProdOrigin::*;
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
            Err(_) => (0, 0_i32),
        };

        let carrier: Carrier = items[baseidx].parse()?;
        let ctype = items[baseidx + 1];
        let csubtype: ProdOrigin = items[baseidx + 2].parse()?;

        // Check coherence of ctype and csubtype
        let subtype_belongs_to_type = match csubtype {
            INSITU => carrier == ELECTRICIDAD || carrier == MEDIOAMBIENTE,
            COGENERACION => carrier == ELECTRICIDAD,
        };
        if !(ctype == "PRODUCCION" && subtype_belongs_to_type) {
            return Err(EpbdError::ParseError(format!(
                "Componente de energía generada con formato incorrecto: {}",
                s
            )));
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

        Ok(ProducedEnergy {
            id,
            carrier,
            csubtype,
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
    fn produced_energy_component() {
        // produced energy component
        let component2 = ProducedEnergy {
            id: 0,
            carrier: "ELECTRICIDAD".parse().unwrap(),
            csubtype: "INSITU".parse().unwrap(),
            service: "NDEF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario prod 1".into(),
        };
        let component2str = "0, ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";
        let component2strlegacy = "ELECTRICIDAD, PRODUCCION, INSITU, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";
        assert_eq!(component2.to_string(), component2str);

        // roundtrip building from/to string
        assert_eq!(
            component2str.parse::<ProducedEnergy>().unwrap().to_string(),
            component2str
        );
        // roundtrip building from/to string for legacy format
        assert_eq!(
            component2strlegacy
                .parse::<ProducedEnergy>()
                .unwrap()
                .to_string(),
            component2str
        );
    }
}
