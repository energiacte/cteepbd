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

use super::{CSubtype, CType, Carrier, Service};
use crate::error::EpbdError;

/// Elements that have a list of numeric values
pub trait HasValues {
    /// Get list of values
    fn values(&self) -> &[f32];

    /// Sum of all values
    fn values_sum(&self) -> f32 {
        self.values().iter().sum::<f32>()
    }

    /// Number of steps
    fn num_steps(&self) -> usize {
        self.values().len()
    }
}

// -------------------- Component
// Define basic Component type

/// Componente de energía.
///
/// Representa la producción o consumo de energía para cada paso de cálculo
/// y a lo largo del periodo de cálculo, para cada tipo, subtipo y uso de la energía.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// System or part id
    /// This can identify the system or part linked to this component.
    /// By default, id=0 means the whole building (all zones, whole building system)
    /// Negative numbers should represent ficticious elements (ficticious zones or systems, such as reference ones)
    /// A value that is not 0 could identify the system that generates or uses some energy
    pub id: i32,
    /// Carrier name
    pub carrier: Carrier,
    /// Component type
    /// - `PRODUCCION` for produced energy components from system Y (E_pr_cr_Y_t, where cr is electricity or ambient energy, could be Q_X_Y_in for solar systems)
    /// - `CONSUMO` for consumed / used energy components for system Y providing service X (E_X_gen_Y_in_cr_t)
    pub ctype: CType,
    /// Energy origin or end use type
    /// - `INSITU` or `COGENERACION` for generated energy component types
    /// - `EPB` or `NEPB` for used energy component types
    pub csubtype: CSubtype,
    /// End use
    pub service: Service,
    /// List of energy values, one value for each timestep. Negative values mean absorbed energy. kWh
    pub values: Vec<f32>,
    /// Descriptive comment string
    /// This can also be used to label a component as auxiliary energy use
    /// by including in this field the "CTEEPBD_AUX" tag
    pub comment: String,
}

impl Component {
    /// Check if component matches a given service
    pub fn has_service(&self, service: Service) -> bool {
        self.service == service
    }

    /// Check if component matches a given carrier
    pub fn has_carrier(&self, carrier: Carrier) -> bool {
        self.carrier == carrier
    }

    /// Check if component has carrier == ELECTRICITY
    pub fn is_electricity(&self) -> bool {
        self.carrier == Carrier::ELECTRICIDAD
    }

    /// Check if component is of generated energy type
    pub fn is_generated(&self) -> bool {
        self.ctype == CType::PRODUCCION
    }

    /// Check if component is of used energy type
    pub fn is_used(&self) -> bool {
        self.ctype == CType::CONSUMO
    }

    /// Check if component is of the epb used energy subtype
    pub fn is_epb(&self) -> bool {
        self.csubtype == CSubtype::EPB
    }
}

impl HasValues for Component {
    fn values(&self) -> &[f32] {
        &self.values
    }
}

impl fmt::Display for Component {
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
            "{}, {}, {}, {}, {}, {}{}",
            self.id, self.carrier, self.ctype, self.csubtype, self.service, valuelist, comment
        )
    }
}

impl str::FromStr for Component {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Component, Self::Err> {
        use self::CSubtype::*;
        use self::CType::*;
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
        let ctype: CType = items[baseidx + 1].parse()?;
        let csubtype: CSubtype = items[baseidx + 2].parse()?;

        // Check coherence of ctype and csubtype
        let subtype_belongs_to_type = match ctype {
            CONSUMO => matches!(csubtype, EPB | NEPB),
            PRODUCCION => match csubtype {
                INSITU => carrier == ELECTRICIDAD || carrier == MEDIOAMBIENTE,
                COGENERACION => carrier == ELECTRICIDAD,
                _ => false,
            }
        };
        if !subtype_belongs_to_type {
            return Err(EpbdError::ParseError(s.into()));
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

        Ok(Component {
            id,
            carrier,
            ctype,
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
    fn tcomponent() {
        // consumer component
        let component1 = Component {
            id: 0,
            carrier: "ELECTRICIDAD".parse().unwrap(),
            ctype: "CONSUMO".parse().unwrap(),
            csubtype: "EPB".parse().unwrap(),
            service: "REF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario cons 1".into(),
        };
        let component1str = "0, ELECTRICIDAD, CONSUMO, EPB, REF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario cons 1";
        assert_eq!(component1.to_string(), component1str);

        // producer component
        let component2 = Component {
            id: 0,
            carrier: "ELECTRICIDAD".parse().unwrap(),
            ctype: "PRODUCCION".parse().unwrap(),
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
            component2str.parse::<Component>().unwrap().to_string(),
            component2str
        );
        // roundtrip building from/to string for legacy format
        assert_eq!(
            component2strlegacy
                .parse::<Component>()
                .unwrap()
                .to_string(),
            component2str
        );
    }
}
