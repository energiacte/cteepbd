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
Energy balance types
====================

Definition of Balance, BalanceForCarrier and BalanceTotal types.

*/

use std::collections::HashMap;

use crate::types::{CSubtype, Carrier, Components, Factors, RenNrenCo2, Service};

/// Detailed results of the energy balance computation for a given carrier
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceForCarrier {
    /// Energy carrier
    pub carrier: Carrier,
    /// Energy used for EPB uses in each timestep
    pub used_EPB: Vec<f32>,
    /// Energy used for EPB uses, by use
    pub used_EPB_an_byuse: HashMap<Service, f32>,
    /// Used energy for non EPB uses in each timestep
    pub used_nEPB: Vec<f32>,
    /// Produced energy in each timestep
    pub produced: Vec<f32>,
    /// Produced energy (from all sources)
    pub produced_an: f32,
    /// Produced energy in each timestep by non grid source (COGENERACION / INSITU)
    pub produced_bygen: HashMap<CSubtype, Vec<f32>>,
    /// Produced energy by non grid source (COGENERACION / INSITU)
    pub produced_bygen_an: HashMap<CSubtype, f32>,
    /// Produced energy from all origins and used for EPB services
    pub produced_used_EPus: Vec<f32>,
    /// Produced energy with origin in generator i and used for EPB services
    pub produced_used_EPus_bygen: HashMap<CSubtype, Vec<f32>>,
    /// Load matching factor
    pub f_match: Vec<f32>,
    /// Exported energy to the grid and non EPB uses in each timestep
    pub exported: Vec<f32>, // exp_used_nEPus + exp_grid
    /// Exported energy to the grid and non EPB uses
    pub exported_an: f32,
    /// Exported energy to the grid and non EPB uses in each timestep, by generation source
    pub exported_bygen: HashMap<CSubtype, Vec<f32>>, // cambiado origin -> gen
    /// Exported energy to the grid and non EPB uses, by generation source
    pub exported_bygen_an: HashMap<CSubtype, f32>, // cambiado origin -> gen
    /// Exported energy to the grid in each timestep
    pub exported_grid: Vec<f32>,
    /// Exported energy to the grid
    pub exported_grid_an: f32,
    /// Exported energy to non EPB uses in each timestep
    pub exported_nEPB: Vec<f32>,
    /// Exported energy to non EPB uses
    pub exported_nEPB_an: f32,
    /// Delivered energy by the grid in each timestep
    pub delivered_grid: Vec<f32>,
    /// Delivered energy by the grid
    pub delivered_grid_an: f32,
    /// Weighted delivered energy by the grid
    pub we_delivered_grid_an: RenNrenCo2,
    /// Weighted delivered energy by any energy production sources
    pub we_delivered_prod_an: RenNrenCo2,
    /// Weighted delivered energy by the grid and any energy production sources
    pub we_delivered_an: RenNrenCo2,
    /// Weighted exported energy for calculation step A
    pub we_exported_an_A: RenNrenCo2,
    /// Weighted exported energy for non EPB uses and calculation step AB
    pub we_exported_nEPB_an_AB: RenNrenCo2,
    /// Weighted exported energy to the grid and calculation step AB
    pub we_exported_grid_an_AB: RenNrenCo2,
    /// Weighted exported energy and calculation step AB
    pub we_exported_an_AB: RenNrenCo2,
    /// Weighted exported energy for calculation step A+B
    pub we_exported_an: RenNrenCo2,
    /// Weighted energy for calculation step A
    pub we_an_A: RenNrenCo2,
    /// Weighted energy for calculation step A, by use (for EPB services)
    pub we_an_A_byuse: HashMap<Service, RenNrenCo2>,
    /// Weighted energy
    pub we_an: RenNrenCo2,
    /// Weighted energy, by use (for EPB services)
    pub we_an_byuse: HashMap<Service, RenNrenCo2>,
}

/// Global balance results (all carriers), either in absolute value or by m2.
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceTotal {
    /// Global energy use for EPB uses, by use
    pub used_EPB_byuse: HashMap<Service, f32>,
    /// Balance result for calculation step A
    pub A: RenNrenCo2,
    /// Weighted energy for calculation step A, by use (for EPB services)
    pub A_byuse: HashMap<Service, RenNrenCo2>,
    /// Balance result for calculation step A+B
    pub B: RenNrenCo2,
    /// Weighted energy, by use (for EPB services)
    pub B_byuse: HashMap<Service, RenNrenCo2>,
    /// Weighted delivered energy
    pub we_del: RenNrenCo2,
    /// Weighted exported energy for calculation step A
    pub we_exp_A: RenNrenCo2,
    /// Weighted exported energy for calculation step A+B
    pub we_exp: RenNrenCo2,
}

/// Data and results of an energy performance computation
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
}
