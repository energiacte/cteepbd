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

/*!
Tipos para el balance energético
================================

Definición de tipos para el balance energético
para la evaluación de la eficiencia energética según la EN ISO 52000-1.

*/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    types::{Carrier, RenNrenCo2, Service, Source},
    Components, Factors,
};

// Overall energy performance
// --------------------------

/// Datos y resultados de un cálculo de eficiencia energética
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Energy components (produced and consumed energy data + metadata)
    pub components: Components,
    /// Weighting factors (weighting factors + metadata)
    pub wfactors: Factors,
    /// Exported energy factor [0, 1]
    pub k_exp: f32,
    /// Reference area used for energy performance ratios (>1e-3)
    pub arearef: f32,
    /// Energy balance results by carrier
    pub balance_cr: HashMap<Carrier, BalanceForCarrier>,
    /// Global energy balance results
    pub balance: BalanceTotal,
    /// Global energy balance results expressed as area ratios
    pub balance_m2: BalanceTotal,
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

/// Resultados del balance global (todos los vectores), en valor absoluto o por m2.
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceTotal {
    /// Energy use for EPB services
    pub used_epus: f32,
    /// Energy use for non EPB services
    pub used_nepus: f32,
    /// Energy use for EPB services, by service
    pub used_epus_by_srv: HashMap<Service, f32>,
    /// Energy use for EPB uses, by carrier
    pub used_epus_by_cr: HashMap<Carrier, f32>,
    /// Produced energy
    pub prod: f32,
    /// Produced energy by source (COGEN / INSITU)
    pub prod_by_src: HashMap<Source, f32>,
    /// Produced energy by carrier
    pub prod_by_cr: HashMap<Carrier, f32>,
    /// Delivered by the grid or onsite sources
    pub del: f32,
    /// Delivered by onsite sources
    pub del_onsite: f32,
    /// Delivered by the grid
    pub del_grid: f32,
    /// Delivered by the grid, by carrier
    pub del_grid_by_cr: HashMap<Carrier, f32>,
    /// Exported energy (to the grid or non EPB services)
    pub exp: f32,
    /// Exported energy to the grid
    pub exp_grid: f32,
    /// Exported energy to nEPB services
    pub exp_nepus: f32,
    /// Balance result for calculation step A
    pub we_a: RenNrenCo2,
    /// Weighted energy for calculation step A, by service (for EPB services)
    pub we_a_by_srv: HashMap<Service, RenNrenCo2>,
    /// Balance result for calculation step A+B
    pub we_b: RenNrenCo2,
    /// Weighted energy, by service (for EPB services)
    pub we_b_by_srv: HashMap<Service, RenNrenCo2>,
    /// Weighted delivered energy
    pub we_del: RenNrenCo2,
    /// Weighted exported energy for calculation step A
    pub we_exp_a: RenNrenCo2,
    /// Weighted exported energy for calculation step A+B
    pub we_exp: RenNrenCo2,
}

impl BalanceTotal {
    /// Normalize values using area
    #[allow(non_snake_case)]
    pub fn normalize_by_area(&self, area: f32) -> BalanceTotal {
        let k_area = if area == 0.0 { 0.0 } else { 1.0 / area };

        let mut used_epus_by_srv = self.used_epus_by_srv.clone();
        used_epus_by_srv.values_mut().for_each(|v| *v *= k_area);

        let mut used_epus_by_cr = self.used_epus_by_cr.clone();
        used_epus_by_cr.values_mut().for_each(|v| *v *= k_area);

        let mut prod_by_src = self.prod_by_src.clone();
        prod_by_src.values_mut().for_each(|v| *v *= k_area);

        let mut prod_by_cr = self.prod_by_cr.clone();
        prod_by_cr.values_mut().for_each(|v| *v *= k_area);

        let mut del_grid_by_cr = self.del_grid_by_cr.clone();
        del_grid_by_cr.values_mut().for_each(|v| *v *= k_area);

        let mut A_by_srv = self.we_a_by_srv.clone();
        A_by_srv.values_mut().for_each(|v| *v *= k_area);

        let mut B_by_srv = self.we_b_by_srv.clone();
        B_by_srv.values_mut().for_each(|v| *v *= k_area);

        BalanceTotal {
            used_epus: k_area * self.used_epus,
            used_nepus: k_area * self.used_nepus,
            used_epus_by_srv,
            used_epus_by_cr,
            prod: k_area * self.prod,
            prod_by_src,
            prod_by_cr,
            del: k_area * self.del,
            del_onsite: k_area * self.del_onsite,
            del_grid: k_area * self.del_grid,
            del_grid_by_cr,
            exp: k_area * self.exp,
            exp_grid: k_area * self.exp_grid,
            exp_nepus: k_area * self.exp_nepus,
            we_a: k_area * self.we_a,
            we_a_by_srv: A_by_srv,
            we_b: k_area * self.we_b,
            we_b_by_srv: B_by_srv,
            we_del: k_area * self.we_del,
            we_exp_a: k_area * self.we_exp_a,
            we_exp: k_area * self.we_exp,
        }
    }
}

impl std::ops::AddAssign<&BalanceForCarrier> for BalanceTotal {
    fn add_assign(&mut self, rhs: &BalanceForCarrier) {
        // Used energy
        self.used_epus += rhs.used.epus_an;
        self.used_nepus += rhs.used.nepus_an;
        // Produced energy
        self.prod += rhs.prod.an;
        // Delivered energy
        self.del += rhs.del.an;
        self.del_onsite += rhs.del.onsite_an;
        self.del_grid += rhs.del.grid_an;
        // Exported energy
        self.exp += rhs.exp.an;
        self.exp_nepus += rhs.exp.used_nepus_an;
        self.exp_grid += rhs.exp.grid_an;

        // Modify global balance using this carrier balance ---
        // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step A
        self.we_a += rhs.we.an_a;
        // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step B
        self.we_b += rhs.we.an;

        // Weighted energy partials
        self.we_del += rhs.we.del_an;
        self.we_exp_a += rhs.we.exp_an_a;
        self.we_exp += rhs.we.exp_an;

        // Aggregation by EPB service
        for (&service, &used_epb_for_service) in &rhs.by_srv.used_epus_an {
            // Energy use
            *self.used_epus_by_srv.entry(service).or_default() += used_epb_for_service;
            // Step A
            if let Some(&value) = rhs.by_srv.we_an_a.get(&service) {
                *self.we_a_by_srv.entry(service).or_default() += value
            }
            // Step B
            if let Some(&value) = rhs.by_srv.we_an.get(&service) {
                *self.we_b_by_srv.entry(service).or_default() += value;
            }
        }

        // Aggregation by energy source
        for (source, produced) in &rhs.prod.by_src_an {
            *self.prod_by_src.entry(*source).or_default() += produced;
        }

        // Aggregation by carrier
        if rhs.prod.an != 0.0 {
            *self.prod_by_cr.entry(rhs.carrier).or_default() += &rhs.prod.an;
        }
        if rhs.del.grid_an != 0.0 {
            *self.del_grid_by_cr.entry(rhs.carrier).or_default() += &rhs.del.grid_an;
        }
        if rhs.used.epus_an != 0.0 {
            *self.used_epus_by_cr.entry(rhs.carrier).or_default() += &rhs.used.epus_an;
        }
    }
}

// Energy balance by carrier
// -------------------------

/// Resultados detallados del balance energético para un vector energético
///
/// Detailed results of the energy balance computation for a given carrier
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceForCarrier {
    /// Energy carrier
    pub carrier: Carrier,
    /// Fraction of used energy for each EPB service
    /// f_us_cr = (used energy for EPB service_i) / (used energy for all EPB services)
    pub f_us: HashMap<Service, f32>,
    /// Load matching factor
    pub f_match: Vec<f32>,
    /// Used energy data and results
    pub used: UsedEnergy,
    /// Produced energy data and results
    pub prod: ProducedEnergy,
    /// Exported energy data and results
    pub exp: ExportedEnergy,
    /// Delivered energy data and results
    pub del: DeliveredEnergy,
    /// Weighted energy data and results
    pub we: WeightedEnergy,
    /// Used and weighted energy, by service
    pub by_srv: ByServiceEnergy,
}

/// Used Energy Data and Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsedEnergy {
    /// Energy used for EPB services at each timestep
    pub epus_t: Vec<f32>,
    /// Energy used for EPB services at each timestep
    pub epus_an: f32,
    /// Used energy for non EPB services at each timestep
    pub nepus_t: Vec<f32>,
    /// Energy used for non EPB services
    pub nepus_an: f32,
}

/// Produced Energy Data and Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducedEnergy {
    /// Produced energy at each timestep
    pub t: Vec<f32>,
    /// Produced energy (from all sources)
    pub an: f32,
    /// Produced energy from all sources and used for EPB services at each timestep
    pub used_epus_t: Vec<f32>,
    /// Produced energy from all sources and used for EPB services
    pub used_epus_an: f32,
    /// Produced energy used for EPB services at each timestep by source (COGEN / INSITU)
    pub used_epus_by_src_t: HashMap<Source, Vec<f32>>,
    /// Produced energy used for EPB services by source (COGEN / INSITU)
    pub used_epus_by_src_an: HashMap<Source, f32>,
    /// Produced energy at each timestep by source (COGEN / INSITU)
    pub by_src_t: HashMap<Source, Vec<f32>>,
    /// Produced energy by source (COGEN / INSITU)
    pub by_src_an: HashMap<Source, f32>,
}

/// Exported Energy Data and Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedEnergy {
    /// Exported energy to the grid and non EPB services at each timestep
    pub t: Vec<f32>, // exp_used_nEPus + exp_grid
    /// Exported energy to the grid and non EPB services
    pub an: f32,
    /// Exported energy to the grid at each timestep
    pub grid_t: Vec<f32>,
    /// Exported energy to the grid
    pub grid_an: f32,
    /// Exported energy to non EPB services at each timestep
    pub used_nepus_t: Vec<f32>,
    /// Exported energy to non EPB services
    pub used_nepus_an: f32,
    /// Exported energy to the grid and non EPB uses at each timestep, by source (INSITU, COGEN)
    pub by_src_t: HashMap<Source, Vec<f32>>,
    /// Exported energy to the grid and non EPB uses, by source (INSITU, COGEN)
    pub by_src_an: HashMap<Source, f32>,
}

/// Delivered Energy Data and Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveredEnergy {
    /// Delivered energy from the grid or onsite sources
    pub an: f32,
    /// Delivered energy by the grid at each timestep
    pub grid_t: Vec<f32>,
    /// Delivered energy by the grid
    pub grid_an: f32,
    /// Delivered energy from onsite sources at each timestep
    pub onsite_t: Vec<f32>,
    /// Delivered energy from onsite sources
    pub onsite_an: f32,
}

/// Weighted Energy Data and Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedEnergy {
    /// Weighted energy
    pub an: RenNrenCo2,
    /// Weighted energy for calculation step A
    pub an_a: RenNrenCo2,
    /// RER (we_an_ren / we_an_tot)
    pub rer: f32,
    /// Weighted delivered energy by the grid and any energy production sources
    pub del_an: RenNrenCo2,
    /// Weighted delivered energy by the grid
    pub del_grid_an: RenNrenCo2,
    /// Weighted delivered energy by any energy production sources
    pub del_onsite_an: RenNrenCo2,
    /// Weighted exported energy for calculation step A+B
    pub exp_an: RenNrenCo2,
    /// Weighted exported energy for calculation step A
    pub exp_an_a: RenNrenCo2,
    /// Weighted exported energy for non EPB services and calculation step AB
    pub exp_nepus_an_ab: RenNrenCo2,
    /// Weighted exported energy to the grid and calculation step AB
    pub exp_grid_an_ab: RenNrenCo2,
    /// Weighted exported energy and calculation step AB
    pub exp_an_ab: RenNrenCo2,
}

/// Used and Weighted Energy results by EPB service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByServiceEnergy {
    /// Energy used for EPB services, by service
    pub used_epus_an: HashMap<Service, f32>,
    /// Weighted energy for calculation step A, by service (for EPB services)
    pub we_an_a: HashMap<Service, RenNrenCo2>,
    /// Weighted energy, by service (for EPB services)
    pub we_an: HashMap<Service, RenNrenCo2>,
}
