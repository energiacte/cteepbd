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
Balance para todos los vectores
===============================

Balance global, con agregación de todos los vectores, en valor absoluto o por m2.
*/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{Carrier, ProdSource, RenNrenCo2, Service};

use super::BalanceCarrier;

/// Resultados del balance global (todos los vectores), en valor absoluto o por m2.
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Balance {
    /// Energy needs (CAL, REF, ACS)
    pub needs: BalNeeds,
    /// Energy use
    pub used: BalUsed,
    /// Produced energy
    pub prod: BalProd,
    /// Delivered energy (by the grid or onsite sources)
    pub del: BalDel,
    /// Exported energy (to the grid or non EPB services)
    pub exp: BalExp,
    /// Weighted energy (step A and B)
    pub we: BalWeighted,
}

impl Balance {
    /// Normalize values using area
    #[allow(non_snake_case)]
    pub fn normalize_by_area(&self, area: f32) -> Balance {
        let k_area = if area == 0.0 { 0.0 } else { 1.0 / area };

        let mut used_epus_by_srv = self.used.epus_by_srv.clone();
        used_epus_by_srv.values_mut().for_each(|v| *v *= k_area);

        let mut used_epus_by_cr = self.used.epus_by_cr.clone();
        used_epus_by_cr.values_mut().for_each(|v| *v *= k_area);

        let mut used_epus_by_srv_by_cr = self.used.epus_by_cr_by_srv.clone();
        used_epus_by_srv_by_cr
            .values_mut()
            .for_each(|v| v.values_mut().for_each(|v| *v *= k_area));

        let mut prod_by_src = self.prod.by_src.clone();
        prod_by_src.values_mut().for_each(|v| *v *= k_area);

        let mut prod_epus_by_src = self.prod.epus_by_src.clone();
        prod_epus_by_src.values_mut().for_each(|v| *v *= k_area);

        let mut prod_epus_by_srv_by_src = self.prod.epus_by_srv_by_src.clone();
        prod_epus_by_srv_by_src
            .values_mut()
            .for_each(|v| v.values_mut().for_each(|v| *v *= k_area));

        let mut prod_by_cr = self.prod.by_cr.clone();
        prod_by_cr.values_mut().for_each(|v| *v *= k_area);

        let mut del_grid_by_cr = self.del.grid_by_cr.clone();
        del_grid_by_cr.values_mut().for_each(|v| *v *= k_area);

        let mut A_by_srv = self.we.a_by_srv.clone();
        A_by_srv.values_mut().for_each(|v| *v *= k_area);

        let mut B_by_srv = self.we.b_by_srv.clone();
        B_by_srv.values_mut().for_each(|v| *v *= k_area);

        Balance {
            needs: BalNeeds {
                ACS: self.needs.ACS.map(|v| v * k_area),
                CAL: self.needs.CAL.map(|v| v * k_area),
                REF: self.needs.REF.map(|v| v * k_area),
            },
            used: BalUsed {
                epus: k_area * self.used.epus,
                nepus: k_area * self.used.nepus,
                cgnus: k_area * self.used.cgnus,
                epus_by_srv: used_epus_by_srv,
                epus_by_cr: used_epus_by_cr,
                epus_by_cr_by_srv: used_epus_by_srv_by_cr,
            },
            prod: BalProd {
                an: k_area * self.prod.an,
                epus_by_src: prod_epus_by_src,
                epus_by_srv_by_src: prod_epus_by_srv_by_src,
                by_src: prod_by_src,
                by_cr: prod_by_cr,
            },
            del: BalDel {
                an: k_area * self.del.an,
                onst: k_area * self.del.onst,
                grid: k_area * self.del.grid,
                grid_by_cr: del_grid_by_cr,
            },
            exp: BalExp {
                an: k_area * self.exp.an,
                grid: k_area * self.exp.grid,
                nepus: k_area * self.exp.nepus,
            },
            we: BalWeighted {
                a: k_area * self.we.a,
                a_by_srv: A_by_srv,
                b: k_area * self.we.b,
                b_by_srv: B_by_srv,
                del: k_area * self.we.del,
                exp_a: k_area * self.we.exp_a,
                exp: k_area * self.we.exp,
            },
        }
    }
}

impl std::ops::AddAssign<&BalanceCarrier> for Balance {
    fn add_assign(&mut self, rhs: &BalanceCarrier) {
        // Used energy
        self.used.epus += rhs.used.epus_an;
        self.used.nepus += rhs.used.nepus_an;
        self.used.cgnus += rhs.used.cgnus_an;
        // Produced energy
        self.prod.an += rhs.prod.an;
        // Delivered energy
        self.del.an += rhs.del.an;
        self.del.onst += rhs.del.onst_an;
        self.del.grid += rhs.del.grid_an;
        // Exported energy
        self.exp.an += rhs.exp.an;
        self.exp.nepus += rhs.exp.nepus_an;
        self.exp.grid += rhs.exp.grid_an;

        // Modify global balance using this carrier balance ---
        // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step A
        self.we.a += rhs.we.a;
        // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step B
        self.we.b += rhs.we.b;

        // Weighted energy partials
        self.we.del += rhs.we.del;
        self.we.exp_a += rhs.we.exp_a;
        self.we.exp += rhs.we.exp;

        // Aggregation by EPB service
        for (&service, &used_epb_for_service) in &rhs.used.epus_by_srv_an {
            // Energy use
            *self.used.epus_by_srv.entry(service).or_default() += used_epb_for_service;
            // Step A
            if let Some(&value) = rhs.we.a_by_srv.get(&service) {
                *self.we.a_by_srv.entry(service).or_default() += value
            }
            // Step B
            if let Some(&value) = rhs.we.b_by_srv.get(&service) {
                *self.we.b_by_srv.entry(service).or_default() += value;
            }
            // By carrier detail
            *self
                .used
                .epus_by_cr_by_srv
                .entry(service)
                .or_default()
                .entry(rhs.carrier)
                .or_default() += used_epb_for_service;
        }

        // Aggregation by energy source
        for (source, produced) in &rhs.prod.by_src_an {
            *self.prod.by_src.entry(*source).or_default() += produced;
        }
        for (source, produced) in &rhs.prod.epus_by_src_an {
            *self.prod.epus_by_src.entry(*source).or_default() += produced;
        }

        for (source, epus_by_srv_for_src) in &rhs.prod.epus_by_srv_by_src_an {
            let hash_srv = self.prod.epus_by_srv_by_src.entry(*source).or_default();
            for (service, epus_for_srv_for_src) in epus_by_srv_for_src {
                *hash_srv.entry(*service).or_default() += epus_for_srv_for_src;
            }
        }

        // Aggregation by carrier
        if rhs.prod.an != 0.0 {
            *self.prod.by_cr.entry(rhs.carrier).or_default() += &rhs.prod.an;
        }
        if rhs.del.grid_an != 0.0 {
            *self.del.grid_by_cr.entry(rhs.carrier).or_default() += &rhs.del.grid_an;
        }
        if rhs.used.epus_an != 0.0 {
            *self.used.epus_by_cr.entry(rhs.carrier).or_default() += &rhs.used.epus_an;
        }
    }
}

/// Demandas del edificio
#[allow(non_snake_case)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BalNeeds {
    /// Building energy needs to provide the domestic heat water service, Q_DHW_nd. kWh
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub ACS: Option<f32>,
    /// Building energy needs to provide the heating service, Q_H_nd. kWh
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub CAL: Option<f32>,
    /// Building energy needs to provide the cooling service, Q_C_nd. kWh
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub REF: Option<f32>,
}

/// Datos de energía consumida para el balance global
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalUsed {
    /// Energy use for non EPB services
    pub nepus: f32,
    /// Energy use for EPB services
    pub epus: f32,
    /// Energy use for Cogen
    pub cgnus: f32,
    /// Energy use for EPB services, by service
    pub epus_by_srv: HashMap<Service, f32>,
    /// Energy use for EPB uses, by carrier
    pub epus_by_cr: HashMap<Carrier, f32>,
    /// Energy use for EPB services, by service, by carrier
    pub epus_by_cr_by_srv: HashMap<Service, HashMap<Carrier, f32>>,
}

/// Datos de energía producida in situ o cogenerada para el balance global
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalProd {
    /// Produced energy from all sources
    pub an: f32,
    /// Produced energy by carrier
    pub by_cr: HashMap<Carrier, f32>,
    /// Produced energy by source
    pub by_src: HashMap<ProdSource, f32>,
    /// Produced energy delivered to EPB services, by source
    pub epus_by_src: HashMap<ProdSource, f32>,
    /// Produced energy delivered for each EPB service, by source
    pub epus_by_srv_by_src: HashMap<ProdSource, HashMap<Service, f32>>,
}

/// Datos de energía suministrada por la red o producción insitu para el balance global
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalDel {
    /// Delivered by the grid or onsite sources
    pub an: f32,
    /// Delivered by onsite sources
    pub onst: f32,
    /// Delivered by the grid
    pub grid: f32,
    /// Delivered by the grid, by carrier
    pub grid_by_cr: HashMap<Carrier, f32>,
}

/// Datos de energía exportada a la red o a usos no EPB para el balance global
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalExp {
    /// Exported energy (to the grid or non EPB services)
    pub an: f32,
    /// Exported energy to the grid
    pub grid: f32,
    /// Exported energy to nEPB services
    pub nepus: f32,
}

/// Datos de energía ponderada, paso A y B para el balance global
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalWeighted {
    /// Balance result for calculation step A
    pub a: RenNrenCo2,
    /// Weighted energy for calculation step A, by EPB service
    pub a_by_srv: HashMap<Service, RenNrenCo2>,
    /// Balance result for calculation step B
    pub b: RenNrenCo2,
    /// Weighted energy, by EPB service
    pub b_by_srv: HashMap<Service, RenNrenCo2>,
    /// Weighted delivered energy for calculation step B
    pub del: RenNrenCo2,
    /// Weighted exported energy for calculation step A
    pub exp_a: RenNrenCo2,
    /// Weighted exported energy for calculation step B
    pub exp: RenNrenCo2,
}
