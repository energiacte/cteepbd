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

Definición de los tipos Balance, BalanceForCarrier and BalanceTotal
y de los métodos que implementan la evaluación de la eficiencia energética
según la EN ISO 52000-1.

*/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::{EpbdError, Result},
    types::HasValues,
    types::{Carrier, Dest, Energy, RenNrenCo2, Service, Source, Step},
    vecops::{veckmul, vecsum, vecvecdif, vecvecmin, vecvecmul, vecvecsum},
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
    pub misc: Option<HashMap<String, String>>,
}

/// Resultados del balance global (todos los vectores), en valor absoluto o por m2.
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceTotal {
    /// Energy use for EPB uses
    pub used_EPB: f32,
    /// Energy use for non EPB uses
    pub used_nEPB: f32,
    /// Energy use for EPB uses, by use
    pub used_EPB_by_service: HashMap<Service, f32>,
    /// Produced energy
    pub produced: f32,
    /// Produced energy by source (COGEN / INSITU)
    pub produced_by_source: HashMap<Source, f32>,
    /// Produced energy by carrier
    pub produced_by_carrier: HashMap<Carrier, f32>,
    /// Delivered  (from the grid)
    pub delivered: f32,
    /// Exported energy (to the grid or nEPB uses)
    pub exported: f32,
    /// Exported energy to the grid
    pub exported_grid: f32,
    /// Exported energy to nEPB uses
    pub exported_nEPB: f32,
    /// Balance result for calculation step A
    pub A: RenNrenCo2,
    /// Weighted energy for calculation step A, by use (for EPB services)
    pub A_by_service: HashMap<Service, RenNrenCo2>,
    /// Balance result for calculation step A+B
    pub B: RenNrenCo2,
    /// Weighted energy, by use (for EPB services)
    pub B_by_service: HashMap<Service, RenNrenCo2>,
    /// Weighted delivered energy
    pub we_del: RenNrenCo2,
    /// Weighted exported energy for calculation step A
    pub we_exp_A: RenNrenCo2,
    /// Weighted exported energy for calculation step A+B
    pub we_exp: RenNrenCo2,
}

impl BalanceTotal {
    /// Normalize values using area
    #[allow(non_snake_case)]
    pub fn normalize_by_area(&self, area: f32) -> BalanceTotal {
        let k_area = if area == 0.0 { 0.0 } else { 1.0 / area };

        let mut used_EPB_by_service = self.used_EPB_by_service.clone();
        used_EPB_by_service.values_mut().for_each(|v| *v *= k_area);

        let mut produced_by_source = self.produced_by_source.clone();
        produced_by_source.values_mut().for_each(|v| *v *= k_area);

        let mut produced_by_carrier = self.produced_by_carrier.clone();
        produced_by_carrier.values_mut().for_each(|v| *v *= k_area);

        let mut A_by_service = self.A_by_service.clone();
        A_by_service.values_mut().for_each(|v| *v *= k_area);

        let mut B_by_service = self.B_by_service.clone();
        B_by_service.values_mut().for_each(|v| *v *= k_area);

        BalanceTotal {
            used_EPB: k_area * self.used_EPB,
            used_nEPB: k_area * self.used_nEPB,
            used_EPB_by_service,
            produced: k_area * self.produced,
            produced_by_source,
            produced_by_carrier,
            delivered: k_area * self.delivered,
            exported: k_area * self.exported,
            exported_grid: k_area * self.exported_grid,
            exported_nEPB: k_area * self.exported_nEPB,
            A: k_area * self.A,
            A_by_service,
            B: k_area * self.B,
            B_by_service,
            we_del: k_area * self.we_del,
            we_exp_A: k_area * self.we_exp_A,
            we_exp: k_area * self.we_exp,
        }
    }
}

/// Calcula enficiencia energética agregando resultados por vector energético
///
/// Compute overall energy performance by aggregating results from all energy carriers.
///
/// * `components` - energy components
/// * `wfactors` - weighting factors
/// * `k_exp` - exported energy factor [0, 1]
/// * `arearef` - reference area used for computing energy performance ratios
///
/// # Errors
///
/// * Use of an `arearef` less than 1e-3 raises an error
/// * Missing weighting factors needed for balance computation
///
#[allow(non_snake_case)]
pub fn energy_performance(
    components: &Components,
    wfactors: &Factors,
    k_exp: f32,
    arearef: f32,
) -> Result<Balance> {
    if arearef < 1e-3 {
        return Err(EpbdError::WrongInput(format!(
            "El área de referencia no puede ser nula o casi nula y se encontró {}",
            arearef
        )));
    };

    // Compute balance for each carrier and accumulate partial balance values for total balance
    let mut balance = BalanceTotal::default();
    let mut balance_cr: HashMap<Carrier, BalanceForCarrier> = HashMap::new();
    for cr in &components.available_carriers() {
        // Compute balance for this carrier ---
        let bal_cr = balance_for_carrier(*cr, components, wfactors, k_exp)?;

        // Used energy
        balance.used_EPB += bal_cr.used.EPB_an;
        balance.used_nEPB += bal_cr.used.nEPB_an;
        // Produced energy
        balance.produced += bal_cr.prod.an;
        // Delivered energy
        balance.delivered += bal_cr.del.grid_an;
        // Exported energy
        balance.exported += bal_cr.exp.an;
        balance.exported_nEPB += bal_cr.exp.used_nEPB_an;
        balance.exported_grid += bal_cr.exp.grid_an;

        // Modify global balance using this carrier balance ---
        // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step A
        balance.A += bal_cr.we.an_A;
        // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step B
        balance.B += bal_cr.we.an;

        // Weighted energy partials
        balance.we_del += bal_cr.we.del_an;
        balance.we_exp_A += bal_cr.we.exp_an_A;
        balance.we_exp += bal_cr.we.exp_an;

        // Aggregation by EPB service
        for (&service, &used_EPB_for_service) in &bal_cr.by_service.used_EPB_an {
            // Energy use
            *balance.used_EPB_by_service.entry(service).or_default() += used_EPB_for_service;
            // Step A
            if let Some(&value) = bal_cr.by_service.we_an_A.get(&service) {
                *balance.A_by_service.entry(service).or_default() += value
            }
            // Step B
            if let Some(&value) = bal_cr.by_service.we_an.get(&service) {
                *balance.B_by_service.entry(service).or_default() += value;
            }
        }

        // Aggregation by energy source
        for (source, produced) in &bal_cr.prod.by_source_an {
            *balance.produced_by_source.entry(*source).or_default() += produced;
        }

        // Aggregation by carrier
        if bal_cr.prod.an != 0.0 {
            *balance.produced_by_carrier.entry(*cr).or_default() += &bal_cr.prod.an;
        }

        // Append to the map of balances by carrier ---
        balance_cr.insert(*cr, bal_cr);
    }

    // Compute area weighted total balance
    let balance_m2 = balance.normalize_by_area(arearef);

    // Global data and results
    Ok(Balance {
        components: components.clone(),
        wfactors: wfactors.clone(),
        k_exp,
        arearef,
        balance_cr,
        balance,
        balance_m2,
        misc: None,
    })
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
    pub by_service: ByServiceEnergy,
}

/// Used Energy Data and Results
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsedEnergy {
    /// Energy used for EPB uses at each timestep
    pub EPB_t: Vec<f32>,
    /// Energy used for EPB uses at each timestep
    pub EPB_an: f32,
    /// Used energy for non EPB uses at each timestep
    pub nEPB_t: Vec<f32>,
    /// Energy used for non EPB uses
    pub nEPB_an: f32,
}

/// Produced Energy Data and Results
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducedEnergy {
    /// Produced energy at each timestep
    pub t: Vec<f32>,
    /// Produced energy (from all sources)
    pub an: f32,
    /// Produced energy from all sources and used for EPB services at each timestep
    pub used_EPus_t: Vec<f32>,
    /// Produced energy from all sources and used for EPB services
    pub used_EPus_an: f32,
    /// Produced energy used for EPB services at each timestep by source (COGEN / INSITU)
    pub used_EPus_by_source_t: HashMap<Source, Vec<f32>>,
    /// Produced energy used for EPB services by source (COGEN / INSITU)
    pub used_EPus_by_source_an: HashMap<Source, f32>,
    /// Produced energy at each timestep by source (COGEN / INSITU)
    pub by_source_t: HashMap<Source, Vec<f32>>,
    /// Produced energy by source (COGEN / INSITU)
    pub by_source_an: HashMap<Source, f32>,
}

/// Exported Energy Data and Results
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedEnergy {
    /// Exported energy to the grid and non EPB uses at each timestep
    pub t: Vec<f32>, // exp_used_nEPus + exp_grid
    /// Exported energy to the grid and non EPB uses
    pub an: f32,
    /// Exported energy to the grid at each timestep
    pub grid_t: Vec<f32>,
    /// Exported energy to the grid
    pub grid_an: f32,
    /// Exported energy to non EPB uses at each timestep
    pub used_nEPB_t: Vec<f32>,
    /// Exported energy to non EPB uses
    pub used_nEPB_an: f32,
    /// Exported energy to the grid and non EPB uses at each timestep, by source (INSITU, COGEN)
    pub by_source_t: HashMap<Source, Vec<f32>>,
    /// Exported energy to the grid and non EPB uses, by source (INSITU, COGEN)
    pub by_source_an: HashMap<Source, f32>,
}

/// Delivered Energy Data and Results
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveredEnergy {
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
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedEnergy {
    /// Weighted energy
    pub an: RenNrenCo2,
    /// Weighted energy for calculation step A
    pub an_A: RenNrenCo2,
    /// RER (we_an_ren / we_an_tot)
    pub rer: f32,
    /// Weighted delivered energy by the grid and any energy production sources
    pub del_an: RenNrenCo2,
    /// Weighted delivered energy by the grid
    pub del_grid_an: RenNrenCo2,
    /// Weighted delivered energy by any energy production sources
    pub del_prod_an: RenNrenCo2,
    /// Weighted exported energy for calculation step A+B
    pub exp_an: RenNrenCo2,
    /// Weighted exported energy for calculation step A
    pub exp_an_A: RenNrenCo2,
    /// Weighted exported energy for non EPB uses and calculation step AB
    pub exp_nEPB_an_AB: RenNrenCo2,
    /// Weighted exported energy to the grid and calculation step AB
    pub exp_grid_an_AB: RenNrenCo2,
    /// Weighted exported energy and calculation step AB
    pub exp_an_AB: RenNrenCo2,
}

/// Used and Weighted Energy results by EPB service
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByServiceEnergy {
    /// Energy used for EPB uses, by use
    pub used_EPB_an: HashMap<Service, f32>,
    /// Weighted energy for calculation step A, by use (for EPB services)
    pub we_an_A: HashMap<Service, RenNrenCo2>,
    /// Weighted energy, by use (for EPB services)
    pub we_an: HashMap<Service, RenNrenCo2>,
}

// --------------------------------------------------------------------
// Energy calculation functions
// --------------------------------------------------------------------

// ///////////// By Carrier timestep and annual computations ////////////

/// Calcula balance energético para un vector energético
///
/// Calculate energy balance for carrier.
///
/// This follows the ISO EN 52000-1 procedure for calculation of delivered,
/// exported and weighted energy balance.
///
/// * `cr_list` - list of components for carrier
/// * `k_exp` - exported energy factor [0, 1]
/// * `fp_cr` - weighting factors for carrier
///
/// # Errors
///
/// * Missing weighting factors for a carrier, source type, destination or calculation step
#[allow(non_snake_case)]
fn balance_for_carrier(
    carrier: Carrier,
    components: &Components,
    wfactors: &Factors,
    k_exp: f32,
) -> Result<BalanceForCarrier> {
    let cr_list: Vec<Energy> = components
        .cdata
        .iter()
        .filter(|e| e.has_carrier(carrier))
        .cloned()
        .collect();

    // Compute fraction of used energy for each EPB service:
    // f_us_cr = (used energy for service_i) / (used energy for all services)
    let f_us_cr = compute_factors_by_use_cr(&cr_list);

    // Compute used and produced energy from components
    let (used, prod, f_match) = compute_used_produced(cr_list);

    // Compute exported and delivered energy from used and produced energy data
    let (exp, del) = compute_exported_delivered(&used, &prod);

    let we = compute_weighted_energy(carrier, k_exp, wfactors, &exp, &del)?;

    let by_service = distribute_by_service(&used, &we, &f_us_cr);

    Ok(BalanceForCarrier {
        carrier,
        f_us: f_us_cr,
        f_match,
        used,
        prod,
        exp,
        del,
        we,
        by_service,
    })
}

/// Compute used and produced energy data from energy components
///
/// TODO:
/// - Implementar uso de baterías (almacenamiento, sto)
/// - Implementar factor de reparto de carga f_match_t
///   Ver ISO_DIS_52000-1_SS_2015_05_13.xlsm
///     - si pr/us <=0; f_match_t = 1
///     - si pr/us > 0; f_match_t = (pr/us + 1/pr/us - 1) / (pr/us + 1/pr/us)
#[allow(non_snake_case)]
fn compute_used_produced(cr_list: Vec<Energy>) -> (UsedEnergy, ProducedEnergy, Vec<f32>) {
    // We know all carriers have the same timesteps (see FromStr for Components)
    let num_steps = cr_list[0].num_steps();

    let mut E_EPus_cr_t = vec![0.0; num_steps];
    let mut E_nEPus_cr_t = vec![0.0; num_steps];
    let mut E_pr_cr_j_t = HashMap::<Source, Vec<f32>>::new();
    for c in &cr_list {
        if c.is_used() || c.is_aux() {
            if c.is_epb_use() {
                E_EPus_cr_t = vecvecsum(&E_EPus_cr_t, c.values())
            } else {
                E_nEPus_cr_t = vecvecsum(&E_nEPus_cr_t, c.values())
            }
        } else if c.is_generated() {
            E_pr_cr_j_t
                .entry(c.source())
                .and_modify(|e| *e = vecvecsum(e, c.values()))
                .or_insert_with(|| c.values().to_owned());
        }
    }
    let E_EPus_cr_an = vecsum(&E_EPus_cr_t);
    let E_nEPus_cr_an = vecsum(&E_nEPus_cr_t);
    let prod_sources: Vec<Source> = E_pr_cr_j_t.keys().cloned().collect();
    let mut E_pr_cr_j_an = HashMap::<Source, f32>::new();
    for source in &prod_sources {
        E_pr_cr_j_an.insert(*source, vecsum(&E_pr_cr_j_t[source]));
    }
    let mut E_pr_cr_t = vec![0.0; num_steps];
    for source in &prod_sources {
        E_pr_cr_t = vecvecsum(&E_pr_cr_t, &E_pr_cr_j_t[source])
    }
    let E_pr_cr_an = vecsum(&E_pr_cr_t);

    // TODO: implement optional computation of f_match_t
    let f_match_t = vec![1.0; num_steps];

    let E_pr_cr_used_EPus_t = vecvecmul(&f_match_t, &vecvecmin(&E_EPus_cr_t, &E_pr_cr_t));
    let E_pr_cr_used_EPus_an = vecsum(&E_pr_cr_used_EPus_t);
    let mut E_pr_cr_j_used_EPus_t = HashMap::<Source, Vec<f32>>::new();

    for source in &prod_sources {
        // * Fraction of produced energy from source j (formula 14)
        // We have grouped by source type (it could be made by generator i, for each one of them)
        let f_pr_cr_j = if E_pr_cr_an > 1e-3 {
            E_pr_cr_j_an[source] / E_pr_cr_an
        } else {
            0.0
        };

        E_pr_cr_j_used_EPus_t.insert(*source, veckmul(&E_pr_cr_used_EPus_t, f_pr_cr_j));
    }
    let E_pr_cr_j_used_EPus_an: HashMap<Source, f32> = E_pr_cr_j_used_EPus_t
        .iter()
        .map(|(source, values)| (*source, vecsum(values)))
        .collect();
    (
        UsedEnergy {
            EPB_t: E_EPus_cr_t,
            EPB_an: E_EPus_cr_an,
            nEPB_t: E_nEPus_cr_t,
            nEPB_an: E_nEPus_cr_an,
        },
        ProducedEnergy {
            t: E_pr_cr_t,
            an: E_pr_cr_an,
            by_source_t: E_pr_cr_j_t,
            by_source_an: E_pr_cr_j_an,
            used_EPus_t: E_pr_cr_used_EPus_t,
            used_EPus_an: E_pr_cr_used_EPus_an,
            used_EPus_by_source_t: E_pr_cr_j_used_EPus_t,
            used_EPus_by_source_an: E_pr_cr_j_used_EPus_an,
        },
        f_match_t,
    )
}

/// Compute exported and delivered energy from used and produced energy data
#[allow(non_snake_case)]
fn compute_exported_delivered(
    used: &UsedEnergy,
    prod: &ProducedEnergy,
) -> (ExportedEnergy, DeliveredEnergy) {
    let E_exp_cr_t = vecvecdif(&prod.t, &prod.used_EPus_t);
    let E_exp_cr_used_nEPus_t = vecvecmin(&E_exp_cr_t, &used.nEPB_t);
    let E_exp_cr_used_nEPus_an = vecsum(&E_exp_cr_used_nEPus_t);
    let E_exp_cr_grid_t = vecvecdif(&E_exp_cr_t, &E_exp_cr_used_nEPus_t);
    let E_exp_cr_grid_an = vecsum(&E_exp_cr_grid_t);
    let E_del_cr_t = vecvecdif(&used.EPB_t, &prod.used_EPus_t);
    let E_del_cr_an = vecsum(&E_del_cr_t);
    let E_del_cr_onsite_t = prod
        .by_source_t
        .get(&Source::INSITU)
        .cloned()
        .unwrap_or_else(|| vec![0.0_f32; E_del_cr_t.len()]);
    let E_del_cr_onsite_an = vecsum(&E_del_cr_onsite_t);
    let mut E_exp_cr_j_t = HashMap::<Source, Vec<f32>>::new();
    for (source, values) in &prod.by_source_t {
        E_exp_cr_j_t.insert(
            *source,
            vecvecdif(values, &prod.used_EPus_by_source_t[source]),
        );
    }
    let mut E_exp_cr_j_an = HashMap::<Source, f32>::new();
    for (source, values) in &E_exp_cr_j_t {
        E_exp_cr_j_an.insert(*source, vecsum(values));
    }
    let E_exp_cr_an = E_exp_cr_used_nEPus_an + E_exp_cr_grid_an;

    (
        ExportedEnergy {
            t: E_exp_cr_t, // exp_used_nEPus + exp_grid
            an: E_exp_cr_an,
            by_source_t: E_exp_cr_j_t,
            by_source_an: E_exp_cr_j_an,
            grid_t: E_exp_cr_grid_t,
            grid_an: E_exp_cr_grid_an,
            used_nEPB_t: E_exp_cr_used_nEPus_t,
            used_nEPB_an: E_exp_cr_used_nEPus_an,
        },
        DeliveredEnergy {
            grid_t: E_del_cr_t,
            grid_an: E_del_cr_an,
            onsite_t: E_del_cr_onsite_t,
            onsite_an: E_del_cr_onsite_an,
        },
    )
}

/// Compute weighted energy from exported and delivered data
#[allow(non_snake_case)]
fn compute_weighted_energy(
    carrier: Carrier,
    k_exp: f32,
    wfactors: &Factors,
    exp: &ExportedEnergy,
    del: &DeliveredEnergy,
) -> Result<WeightedEnergy> {
    let E_we_del_cr_grid_an =
        del.grid_an * wfactors.find(carrier, Source::RED, Dest::SUMINISTRO, Step::A)?;
    let E_we_del_cr_onsite_an = if del.onsite_an == 0.0 {
        RenNrenCo2::default()
    } else {
        del.onsite_an * wfactors.find(carrier, Source::INSITU, Dest::SUMINISTRO, Step::A)?
    };
    let E_we_del_cr_an = E_we_del_cr_grid_an + E_we_del_cr_onsite_an;

    let mut E_we_exp_cr_an_A = RenNrenCo2::default();
    let mut E_we_exp_cr_an_AB = RenNrenCo2::default();
    let mut E_we_exp_cr_an = RenNrenCo2::default();
    let mut E_we_exp_cr_used_nEPus_an_AB = RenNrenCo2::default();
    let mut E_we_exp_cr_grid_an_AB = RenNrenCo2::default();
    if exp.an != 0.0 {
        // This case implies there is exported energy.
        // If there's no exportation, it's either because the carrier cannot be exported
        // or because there's no effective exportation
        // * Step A: weighting depends on exported energy generation (by source)
        // Factors are averaged weighting by the amount of production from each source relative to the amount for all sources (no priority, 9.6.6.2.4, eq (8))

        // Compute mean energy weighting factor for all (non grid) sources
        // uses exported energy from source j relative to all exported energy as weighting criteria
        let f_we_exp_cr_compute = |dest: Dest, step: Step| -> Result<RenNrenCo2> {
            let mut result = RenNrenCo2::default();
            for (source, E_exp_cr_gen_an) in &exp.by_source_an {
                let fp_j = wfactors.find(carrier, *source, dest, step)?;
                result += fp_j * (E_exp_cr_gen_an / exp.an);
            }
            Ok(result)
        };

        // Weighting factors for energy exported to nEP uses (step A) (~formula 24)
        let f_we_exp_cr_stepA_nEPus: RenNrenCo2 = if exp.used_nEPB_an == 0.0 {
            // No exported energy to nEP uses
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_NEPB, Step::A)?
        };

        // Weighting factors for energy exported to the grid (step A) (~formula 25)
        let f_we_exp_cr_stepA_grid: RenNrenCo2 = if exp.grid_an == 0.0 {
            // No energy exported to grid
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_RED, Step::A)?
        };

        // Weighted exported energy according to resources used to generate that energy (formula 23)
        E_we_exp_cr_an_A = (exp.used_nEPB_an * f_we_exp_cr_stepA_nEPus) // formula 24
            + (exp.grid_an * f_we_exp_cr_stepA_grid); // formula 25

        // * Step B: weighting depends on exported energy generation and avoided resources on the grid

        // Factors of contribution for energy exported to nEP uses (step B)
        let f_we_exp_cr_used_nEPus = if exp.used_nEPB_an == 0.0 {
            // No energy exported to nEP uses
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_NEPB, Step::B)?
        };

        // Weighting factors for energy exported to the grid (step B)
        let f_we_exp_cr_grid = if exp.grid_an == 0.0 {
            // No energy exported to grid
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_RED, Step::B)?
        };

        // Effect of exported energy on weighted energy performance (step B) (formula 26)

        E_we_exp_cr_used_nEPus_an_AB =
            exp.used_nEPB_an * (f_we_exp_cr_used_nEPus - f_we_exp_cr_stepA_nEPus);

        E_we_exp_cr_grid_an_AB = exp.grid_an * (f_we_exp_cr_grid - f_we_exp_cr_stepA_grid);

        E_we_exp_cr_an_AB = E_we_exp_cr_used_nEPus_an_AB + E_we_exp_cr_grid_an_AB;

        // Contribution of exported energy to the annual weighted energy performance
        // 11.6.2.1, 11.6.2.2, 11.6.2.3
        E_we_exp_cr_an = E_we_exp_cr_an_A + (k_exp * E_we_exp_cr_an_AB); // (formula 20)
    }
    let E_we_cr_an_A: RenNrenCo2 = E_we_del_cr_an - E_we_exp_cr_an_A;
    let E_we_cr_an: RenNrenCo2 = E_we_del_cr_an - E_we_exp_cr_an;

    Ok(WeightedEnergy {
        an: E_we_cr_an,
        rer: E_we_cr_an.rer(),

        an_A: E_we_cr_an_A,

        del_an: E_we_del_cr_an,
        del_grid_an: E_we_del_cr_grid_an,
        del_prod_an: E_we_del_cr_onsite_an,

        exp_an: E_we_exp_cr_an,
        exp_an_A: E_we_exp_cr_an_A,
        exp_an_AB: E_we_exp_cr_an_AB,
        exp_nEPB_an_AB: E_we_exp_cr_used_nEPus_an_AB,
        exp_grid_an_AB: E_we_exp_cr_grid_an_AB,
    })
}

/// Distribute used and weighted energy data by EPB service
///
/// Allocate energy to services proportionally to its fraction of used energy over total used energy
#[allow(non_snake_case)]
fn distribute_by_service(
    used: &UsedEnergy,
    we: &WeightedEnergy,
    f_us_cr: &HashMap<Service, f32>,
) -> ByServiceEnergy {
    let mut used_EPB_by_service_an: HashMap<Service, f32> = HashMap::new();
    let mut we_an_A_by_service: HashMap<Service, RenNrenCo2> = HashMap::new();
    let mut we_an_by_service: HashMap<Service, RenNrenCo2> = HashMap::new();
    for service in &Service::SERVICES_EPB {
        let f_us_k_cr = *f_us_cr.get(service).unwrap_or(&0.0f32);
        if f_us_k_cr != 0.0 {
            used_EPB_by_service_an.insert(*service, used.EPB_an * f_us_k_cr);
            we_an_A_by_service.insert(*service, we.an_A * f_us_k_cr);
            we_an_by_service.insert(*service, we.an * f_us_k_cr);
        }
    }

    ByServiceEnergy {
        used_EPB_an: used_EPB_by_service_an,
        we_an_A: we_an_A_by_service,
        we_an: we_an_by_service,
    }
}

/// Calcula fracción de cada uso EPB para un vector energético i
///
/// Compute share of each EPB use for a given carrier i
///
/// It uses the reverse calculation method (E.3.6)
/// * `cr_list` - components list for the selected carrier i
///
fn compute_factors_by_use_cr(cr_list: &[Energy]) -> HashMap<Service, f32> {
    let mut factors_us_k: HashMap<Service, f32> = HashMap::new();
    // Energy use components (EPB uses) for current carrier i
    let cr_use_list = cr_list.iter().filter(|c| c.is_epb_use());
    // Energy use for all EPB services and carrier i (Q_Epus_cr)
    let q_us_all: f32 = cr_use_list.clone().map(HasValues::values_sum).sum();
    if q_us_all != 0.0 {
        // No energy use for this carrier!
        // Collect share of step A weighted energy for each use item (service)
        for us in Service::SERVICES_EPB.iter().cloned() {
            // Energy use for use k
            let q_us_k: f32 = cr_use_list
                .clone()
                .filter(|c| c.has_service(us))
                .map(HasValues::values_sum)
                .sum();
            // Factor for use k
            factors_us_k.insert(us, q_us_k / q_us_all);
        }
    }
    factors_us_k
}
