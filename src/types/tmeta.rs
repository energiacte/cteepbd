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
Tipos para la definición de metadatos
=====================================

- Tipo Meta y sus traits
*/

use std::fmt;
use std::str;
use std::str::FromStr;

use crate::{error::EpbdError, types::RenNrenCo2};

// ==================== Metadata types

/// Metadatos de los componentes o de los factores de paso
/// 
/// Metadata of components or weighting factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    /// metadata name.
    pub key: String,
    /// metadata value
    pub value: String,
}

impl Meta {
    /// Metadata constructor
    pub fn new<T, U>(key: T, value: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl fmt::Display for Meta {
    /// Textual representation of metadata.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#META {}: {}", self.key, self.value)
    }
}

impl std::str::FromStr for Meta {
    type Err = EpbdError;

    fn from_str(s: &str) -> Result<Meta, Self::Err> {
        // Remove start of line with #META or #CTE_
        let items: Vec<&str> = s.trim()[5..].splitn(2, ':').map(str::trim).collect();
        if items.len() == 2 {
            let key = match items[0].trim() {
                // Fix legacy values
                "Localizacion" => "CTE_LOCALIZACION",
                "Area_ref" => "CTE_AREAREF",
                "kexp" => "CTE_KEXP",
                x => x,
            };
            let value = items[1].trim();
            Ok(Meta::new(key, value))
        } else {
            Err(EpbdError::ParseError(s.into()))
        }
    }
}

// == Data + Metadata Types ==

/// Trait común para gestionar metadatos
pub trait MetaVec {
    /// Get vector of metadata
    fn get_metavec(&self) -> &Vec<Meta>;

    /// Get mutable vector of metadata
    fn get_mut_metavec(&mut self) -> &mut Vec<Meta>;

    /// Check if key is included in metadata
    fn has_meta(&self, key: &str) -> bool {
        self.get_metavec().iter().any(|m| m.key == key)
    }

    /// Check if key has the given value
    fn has_meta_value(&self, key: &str, value: &str) -> bool {
        self.get_meta(key).map(|v| v == value).unwrap_or(false)
    }

    /// Get (optional) metadata value by key
    fn get_meta(&self, key: &str) -> Option<String> {
        self.get_metavec()
            .iter()
            .find(|m| m.key == key)
            .and_then(|v| Some(v.value.clone()))
    }

    /// Get (optional) metadata value (f32) by key as f32
    fn get_meta_f32(&self, key: &str) -> Option<f32> {
        self.get_metavec()
            .iter()
            .find(|m| m.key == key)
            .and_then(|v| f32::from_str(v.value.trim()).ok())
    }

    /// Get (optional) metadata value (f32, f32) by key as RenNrenCo2 struct
    fn get_meta_rennren(&self, key: &str) -> Option<RenNrenCo2> {
        self.get_metavec()
            .iter()
            .find(|m| m.key == key)
            .and_then(|v| {
                v.value
                    .parse::<RenNrenCo2>()
                    .map_err(|e| {
                        eprintln!("No se puede transformar el metadato a RenNrenCo2: {:?}", v);
                        e
                    })
                    .ok()
            })
    }

    /// Update metadata value for key or insert new metadata.
    fn set_meta(&mut self, key: &str, value: &str) {
        let wmeta = self.get_mut_metavec();
        let metapos = wmeta.iter().position(|m| m.key == key);
        if let Some(pos) = metapos {
            wmeta[pos].value = value.to_string();
        } else {
            wmeta.push(Meta::new(key, value));
        };
    }
}

// ========================== Tests

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq};

    #[test]
    fn tmeta() {
        let meta = Meta {
            key: "CTE_FUENTE".to_string(),
            value: "RITE2014".to_string(),
        };
        let meta2 = Meta::new("CTE_FUENTE", "RITE2014");
        let metastr = "#META CTE_FUENTE: RITE2014";
        assert_eq!(format!("{}", meta), metastr);
        assert_eq!(format!("{}", meta2), metastr);
        assert_eq!(format!("{}", metastr.parse::<Meta>().unwrap()), metastr);
    }
}
