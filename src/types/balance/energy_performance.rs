// Copyright (c) 2018-2023  Ministerio de Fomento
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
Tipos para la eficiencia energética
===================================

Definición de tipos para la evaluación de la eficiencia energética y sus datos,
según la EN ISO 52000-1.

*/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{types::Carrier, Components, Factors};

use super::{BalanceCarrier, Balance};

// Overall energy performance
// --------------------------

/// Datos y resultados de un cálculo de eficiencia energética
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyPerformance {
    /// Energy components (produced and consumed energy data + metadata)
    pub components: Components,
    /// Weighting factors (weighting factors + metadata)
    pub wfactors: Factors,
    /// Exported energy factor [0, 1]
    pub k_exp: f32,
    /// Reference area used for energy performance ratios (>1e-3)
    pub arearef: f32,
    /// Energy balance results by carrier
    pub balance_cr: HashMap<Carrier, BalanceCarrier>,
    /// Global energy balance results
    pub balance: Balance,
    /// Global energy balance results expressed as area ratios
    pub balance_m2: Balance,
    /// Renewable Energy Ratio considering the distant perimeter
    /// RER = we_ren / we_tot
    pub rer: f32,
    /// Renewable Energy Ratio considering onsite and nearby perimeter
    /// RER_nrb = we_ren_nrb+onst / we_tot
    pub rer_nrb: f32,
    /// Renewable Energy Ratio considering onsite perimeter
    /// RER_onst = we_ren_onst / we_tot
    pub rer_onst: f32,
    /// Generic miscelaneous user provided data
    pub misc: Option<MiscMap>,
}

/// Diccionario de valores adicionales
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MiscMap(pub HashMap<String, String>);

impl MiscMap {
    /// Get value as a string with 1 digit precision or a dash if value is missing or is not a number
    pub fn get_str_1d(&self, key: &str) -> String {
        self.get(key)
            .and_then(|v| v.parse::<f32>().map(|r| format!("{:.1}", r)).ok())
            .unwrap_or_else(|| "-".to_string())
    }

    /// Get value as a string for a value, as a percent with 1 digit precision or a dash if value is missing or is not a number
    pub fn get_str_pct1d(&self, key: &str) -> String {
        self.get(key)
            .and_then(|v| v.parse::<f32>().map(|r| format!("{:.1}", 100.0 * r)).ok())
            .unwrap_or_else(|| "-".to_string())
    }
}

impl std::ops::Deref for MiscMap {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MiscMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
