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
Balance para un vector energético
=================================

*/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{Carrier, ProdSource, RenNrenCo2, Service};

// Energy balance by carrier
// -------------------------

/// Resultados detallados del balance energético para un vector energético
///
/// Detailed results of the energy balance computation for a given carrier
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceCarrier {
    /// Energy carrier
    pub carrier: Carrier,
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
}

/// Used Energy Data and Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsedEnergy {
    /// Energy used for EPB services at each timestep
    pub epus_t: Vec<f32>,
    /// Energy used for EPB services at each timestep, by service
    pub epus_by_srv_t: HashMap<Service, Vec<f32>>,
    /// Energy used for EPB services at each timestep
    pub epus_an: f32,
    /// Energy used for EPB services, by service
    pub epus_by_srv_an: HashMap<Service, f32>,
    /// Used energy for non EPB services at each timestep
    pub nepus_t: Vec<f32>,
    /// Energy used for non EPB services
    pub nepus_an: f32,
    /// Energy input allocated to electricity cogeneration at each timestep
    pub cgn_in_t: Vec<f32>,
    /// Energy input allocated to electricity cogeneration
    pub cgn_in_an: f32,
}

/// Produced Energy Data and Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducedEnergy {
    /// Produced energy at each timestep
    pub t: Vec<f32>,
    /// Produced energy (from all sources)
    pub an: f32,
    /// Produced energy at each timestep by source
    pub by_src_t: HashMap<ProdSource, Vec<f32>>,
    /// Produced energy by source
    pub by_src_an: HashMap<ProdSource, f32>,
    /// Produced energy from all sources and used for EPB services at each timestep
    pub epus_t: Vec<f32>,
    /// Produced energy from all sources and used for EPB services
    pub epus_an: f32,
    /// Produced energy used for EPB services at each timestep by source
    pub epus_by_src_t: HashMap<ProdSource, Vec<f32>>,
    /// Produced energy used for EPB services by source
    pub epus_by_src_an: HashMap<ProdSource, f32>,
    /// Produced energy used for EPB services at each timestep by service, by source
    pub epus_by_srv_by_src_t: HashMap<ProdSource, HashMap<Service, Vec<f32>>>,
    /// Produced energy used for EPB services by service, by source
    pub epus_by_srv_by_src_an: HashMap<ProdSource, HashMap<Service, f32>>,
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
    pub nepus_t: Vec<f32>,
    /// Exported energy to non EPB services
    pub nepus_an: f32,
    /// Exported energy to the grid and non EPB services at each timestep, by source
    pub by_src_t: HashMap<ProdSource, Vec<f32>>,
    /// Exported energy to the grid and non EPB services, by source
    pub by_src_an: HashMap<ProdSource, f32>,
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
    pub onst_t: Vec<f32>,
    /// Delivered energy from onsite sources
    pub onst_an: f32,
    /// Delivered energy allocated to electricity cogeneration at each timestep
    pub cgn_t: Vec<f32>,
    /// Delivered energy allocated to electricity cogeneration
    pub cgn_an: f32,
}

/// Weighted Energy Data and Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedEnergy {
    /// Weighted energy for calculation step B
    pub b: RenNrenCo2,
    /// Weighted energy for calculation step B, by service (for EPB services)
    pub b_by_srv: HashMap<Service, RenNrenCo2>,
    /// Weighted energy for calculation step A
    pub a: RenNrenCo2,
    /// Weighted energy for calculation step A, by service (for EPB services)
    pub a_by_srv: HashMap<Service, RenNrenCo2>,
    /// Weighted delivered energy by the grid and any energy production sources
    pub del: RenNrenCo2,
    /// Weighted delivered energy by the grid
    pub del_grid: RenNrenCo2,
    /// Weighted delivered energy by any onsite energy production source (EL_INSITU, TERMOSOLAR, EAMBIENTE)
    pub del_onst: RenNrenCo2,
    /// Weighted delivered energy by cogenerated electricity (EL_COGEN)
    pub del_cgn: RenNrenCo2,
    /// Weighted exported energy for calculation step A+B
    pub exp: RenNrenCo2,
    /// Weighted exported energy for calculation step A (resources used)
    pub exp_a: RenNrenCo2,
    /// Weighted exported energy for non EPB services for calculation step A (resources used)
    pub exp_nepus_a: RenNrenCo2,
    /// Weighted exported energy to the grid and calculation step A (resources used)
    pub exp_grid_a: RenNrenCo2,
    /// Weighted exported energy for non EPB services and calculation step AB
    pub exp_nepus_ab: RenNrenCo2,
    /// Weighted exported energy to the grid and calculation step AB
    pub exp_grid_ab: RenNrenCo2,
    /// Weighted exported energy and calculation step AB
    pub exp_ab: RenNrenCo2,
}
