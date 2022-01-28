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
    types::{
        Carrier, Dest, EnergyData, Factor, ProdSource, RenNrenCo2, Service, Source, Step, SERVICES,
    },
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
        let bal_cr = balance_for_carrier(*cr, &components.cdata, wfactors, k_exp)?;

        // Modify global balance using this carrier balance ---
        // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step A
        balance.A += bal_cr.we_an_A;
        // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step B
        balance.B += bal_cr.we_an;
        // Weighted energy partials
        balance.we_del += bal_cr.we_delivered_an;
        balance.we_exp_A += bal_cr.we_exported_an_A;
        balance.we_exp += bal_cr.we_exported_an;
        // Weighted energy for each use item (EPB services)
        for &service in &SERVICES {
            // Energy use
            if let Some(value) = bal_cr.used_EPB_an_byuse.get(&service) {
                *balance.used_EPB_byuse.entry(service).or_default() += *value
            }
            // Step A
            if let Some(value) = bal_cr.we_an_A_byuse.get(&service) {
                *balance.A_byuse.entry(service).or_default() += *value
            }
            // Step B
            if let Some(value) = bal_cr.we_an_byuse.get(&service) {
                *balance.B_byuse.entry(service).or_default() += *value;
            }
        }

        // Append to the map of balances by carrier ---
        balance_cr.insert(*cr, bal_cr);
    }

    // Compute area weighted total balance
    let k_area = 1.0 / arearef;
    let mut used_EPB_byuse = balance.used_EPB_byuse.clone();
    used_EPB_byuse.values_mut().for_each(|v| *v *= k_area);

    let mut A_byuse = balance.A_byuse.clone();
    A_byuse.values_mut().for_each(|v| *v *= k_area);

    let mut B_byuse = balance.B_byuse.clone();
    B_byuse.values_mut().for_each(|v| *v *= k_area);

    let balance_m2 = BalanceTotal {
        used_EPB_byuse,
        A: k_area * balance.A,
        A_byuse,
        B: k_area * balance.B,
        B_byuse,
        we_del: k_area * balance.we_del,
        we_exp_A: k_area * balance.we_exp_A,
        we_exp: k_area * balance.we_exp,
    };

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
    /// Energy used for EPB uses in each timestep
    pub used_EPB: Vec<f32>,
    /// Energy used for EPB uses, by use
    pub used_EPB_an_byuse: HashMap<Service, f32>,
    /// Used energy for non EPB uses in each timestep
    pub used_nEPB: Vec<f32>,
    /// Energy used for non EPB uses
    pub used_nEPB_an: f32,
    /// Produced energy in each timestep
    pub produced: Vec<f32>,
    /// Produced energy (from all sources)
    pub produced_an: f32,
    /// Produced energy in each timestep by source (COGENERACION / INSITU)
    pub produced_by_source: HashMap<ProdSource, Vec<f32>>,
    /// Produced energy by source (COGENERACION / INSITU)
    pub produced_by_source_an: HashMap<ProdSource, f32>,
    /// Produced energy from all sources and used for EPB services in each timestep
    pub produced_used_EPus: Vec<f32>,
    /// Produced energy from all sources and used for EPB services
    pub produced_used_EPus_an: f32,
    /// Produced energy used for EPB services in each timestep by source (COGENERACION / INSITU)
    pub produced_used_EPus_by_source: HashMap<ProdSource, Vec<f32>>,
    /// Produced energy used for EPB services by source (COGENERACION / INSITU)
    pub produced_used_EPus_by_source_an: HashMap<ProdSource, f32>,
    /// Load matching factor
    pub f_match: Vec<f32>,
    /// Exported energy to the grid and non EPB uses in each timestep
    pub exported: Vec<f32>, // exp_used_nEPus + exp_grid
    /// Exported energy to the grid and non EPB uses
    pub exported_an: f32,
    /// Exported energy to the grid and non EPB uses in each timestep, by source (INSITU, COGENERACION)
    pub exported_by_source: HashMap<ProdSource, Vec<f32>>,
    /// Exported energy to the grid and non EPB uses, by source (INSITU, COGENERACION)
    pub exported_by_source_an: HashMap<ProdSource, f32>,
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
///
/// TODO:
/// - Ahora mismo nosotros consideramos únicamente la existencia de dos orígenes de generación:
///   insitu y cogeneración, sin diferenciar generadores individuales. Es decir, consideramos que hay
///   dos generadores como mucho. La norma hace el reparto de la energía producida (14) por generador i
///   y, si implementamos el soporte generador a generador, habría que revisar esto. En particular,
///   cómo se calcula f_pr_cr_j.
/// - Implementar factor de reparto de carga f_match_t
#[allow(non_snake_case)]
fn balance_for_carrier(
    carrier: Carrier,
    carriers: &[EnergyData],
    wfactors: &Factors,
    k_exp: f32,
) -> Result<BalanceForCarrier> {
    let cr_list: Vec<EnergyData> = carriers
        .iter()
        .filter(|e| e.has_carrier(carrier))
        .cloned()
        .collect();

    let fp_cr: Vec<Factor> = wfactors
        .wdata
        .iter()
        .filter(|e| e.carrier == carrier)
        .cloned()
        .collect();

    // We know all carriers have the same timesteps (see FromStr for Components)
    let num_steps = cr_list[0].num_steps();

    // * Energy used by technical systems for EPB services, for each time step
    let mut E_EPus_cr_t = vec![0.0; num_steps];
    // * Energy used by technical systems for non-EPB services, for each time step
    let mut E_nEPus_cr_t = vec![0.0; num_steps];
    // * Produced on-site energy and inside the assessment boundary, by subsystem j (INSITU or COGENERACION)
    let mut E_pr_cr_j_t = HashMap::<ProdSource, Vec<f32>>::new();

    // Accumulate for all components
    for c in &cr_list {
        if c.is_used() {
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

    // List of produced energy sources (ProdOrigin::INSITU or ProdOrigin::COGENERACION)
    let pr_generators: Vec<ProdSource> = E_pr_cr_j_t.keys().cloned().collect(); // INSITU, COGENERACION

    // Annually produced on-site energy from subsystem j (INSITU, COGENERACION)
    let mut E_pr_cr_j_an = HashMap::<ProdSource, f32>::new();
    for source in &pr_generators {
        E_pr_cr_j_an.insert(*source, vecsum(&E_pr_cr_j_t[source]));
    }

    // * Energy produced on-site and inside the assessment boundary for all generators (formula 30)
    let mut E_pr_cr_t = vec![0.0; num_steps];
    for source in &pr_generators {
        E_pr_cr_t = vecvecsum(&E_pr_cr_t, &E_pr_cr_j_t[source])
    }
    let E_pr_cr_an = vecsum(&E_pr_cr_t);

    // * Produced energy from all sources for EPB services for each time step (formula 31)
    // TODO: f_match_t constant for electricity (formula 32)
    // TODO: let f_match_t = fmatch(E_pr_cr_t / E_EPus_cr_t)
    let f_match_t = vec![1.0; num_steps];

    let E_pr_cr_used_EPus_t = vecvecmul(&f_match_t, &vecvecmin(&E_EPus_cr_t, &E_pr_cr_t));

    // * Exported energy for each time step (produced energy not consumed in EPB uses) (formula 33)
    // E_pr_cr_t = E_pr_cr_used_EPus_t + E_exp_cr_used_nEPus_t + E_exp_cr_grid_t
    // E_exp_cr_t = E_exp_cr_used_nEPus_t + E_exp_cr_grid_t
    // -> E_exp_cr_t = E_pr_cr_t - E_pr_cr_used_EPus_t
    let E_exp_cr_t = vecvecdif(&E_pr_cr_t, &E_pr_cr_used_EPus_t);

    // * Exported energy used for non-EPB uses for each time step (formula 34)
    let E_exp_cr_used_nEPus_t = vecvecmin(&E_exp_cr_t, &E_nEPus_cr_t);

    // * Annualy exported energy used for non-EPB uses for carrier
    let E_exp_cr_used_nEPus_an = vecsum(&E_exp_cr_used_nEPus_t);

    // * Energy exported to the grid for each interval (formula 35)
    let E_exp_cr_grid_t = vecvecdif(&E_exp_cr_t, &E_exp_cr_used_nEPus_t);

    // * Annualy exported energy to the grid for carrier (formula 36)
    let E_exp_cr_grid_an = vecsum(&E_exp_cr_grid_t);

    // * Delivered energy (by the grid) for EP uses for each interval (formula 37)
    let E_del_cr_t = vecvecdif(&E_EPus_cr_t, &E_pr_cr_used_EPus_t);

    // * Annualy delivered energy (by the grid) for EP uses for carrier (formula 38)
    let E_del_cr_an = vecsum(&E_del_cr_t);

    // ** Weighting depending on energy generator **

    // Exported energy by source j (9.6.6.2)
    // Implementation WITHOUT priorities on energy use
    // We are doing this computations using sources, j (INSITU, COGENERACION) not generator ids (i).

    // * Produced energy from source j and used for EPB services (formula 15)
    let mut E_pr_cr_j_used_EPus_t = HashMap::<ProdSource, Vec<f32>>::new();
    for source in &pr_generators {
        // * Fraction of produced energy from source j (formula 14)
        // We have grouped by source type (it could be made by generator i, for each one of them)
        let f_pr_cr_j = if E_pr_cr_an > 1e-3 {
            E_pr_cr_j_an[source] / E_pr_cr_an
        } else {
            0.0
        };

        E_pr_cr_j_used_EPus_t.insert(*source, veckmul(&E_pr_cr_used_EPus_t, f_pr_cr_j));
    }

    // * Exported energy from source j (formula 16)
    let mut E_exp_cr_j_t = HashMap::<ProdSource, Vec<f32>>::new();
    for source in &pr_generators {
        E_exp_cr_j_t.insert(
            *source,
            vecvecdif(&E_pr_cr_j_t[source], &E_pr_cr_j_used_EPus_t[source]),
        );
    }

    // * Annually exported energy from source j
    let mut E_exp_cr_j_an = HashMap::<ProdSource, f32>::new();
    for source in &pr_generators {
        E_exp_cr_j_an.insert(*source, vecsum(&E_exp_cr_j_t[source]));
    }

    // -------- Weighted delivered and exported energy (11.6.2.1, 11.6.2.2, 11.6.2.3 + eq 2, 3)
    // NOTE: All weighting factors have been considered constant through all timesteps
    // NOTE: This allows using annual quantities and not timestep expressions

    // * Weighted energy for delivered energy: the cost of producing that energy
    let fpA_grid = fp_find(&fp_cr, Source::RED, Dest::SUMINISTRO, Step::A)?;
    let E_we_del_cr_grid_an = E_del_cr_an * fpA_grid; // formula 19, 39

    // 2) Delivered energy from non cogeneration on-site sources (source j)
    let E_we_del_cr_onsite_an = E_pr_cr_j_an
        .get(&ProdSource::INSITU)
        .and_then(|E_pr_cr_i| {
            fp_find(&fp_cr, Source::INSITU, Dest::SUMINISTRO, Step::A)
                .map(|fpA_pr_cr_i| E_pr_cr_i * fpA_pr_cr_i)
                .ok()
        })
        .unwrap_or_default();

    // 3) Total delivered energy: grid + all onsite (but non cogeneration)
    let E_we_del_cr_an = E_we_del_cr_grid_an + E_we_del_cr_onsite_an; // formula 19, 39

    // // * Weighted energy for exported energy: depends on step A or B

    let mut E_we_exp_cr_an_A = RenNrenCo2::default();
    let mut E_we_exp_cr_an_AB = RenNrenCo2::default();
    let mut E_we_exp_cr_an = RenNrenCo2::default();
    let mut E_we_exp_cr_used_nEPus_an_AB = RenNrenCo2::default();
    let mut E_we_exp_cr_grid_an_AB = RenNrenCo2::default();

    let E_exp_cr_an = E_exp_cr_used_nEPus_an + E_exp_cr_grid_an;

    if E_exp_cr_an != 0.0 {
        // This case implies there is exported energy.
        // If there's no exportation, it's either because the carrier cannot be exported
        // or because there's no effective exportation
        // * Step A: weighting depends on exported energy generation (by source)
        // Factors are averaged weighting by the amount of production from each source relative to the amount for all sources (no priority, 9.6.6.2.4, eq (8))

        // Compute mean energy weighting factor for all (non grid) sources
        // uses exported energy from source j relative to all exported energy as weighting criteria
        let f_we_exp_cr_compute = |dest: Dest, step: Step| -> Result<RenNrenCo2> {
            let mut result = RenNrenCo2::default();
            for (source, E_exp_cr_gen_an) in &E_exp_cr_j_an {
                let fp_j = fp_find(&fp_cr, *source, dest, step)?;
                result += fp_j * (E_exp_cr_gen_an / E_exp_cr_an);
            }
            Ok(result)
        };

        // Weighting factors for energy exported to nEP uses (step A) (~formula 24)
        let f_we_exp_cr_stepA_nEPus: RenNrenCo2 = if E_exp_cr_used_nEPus_an == 0.0 {
            // No exported energy to nEP uses
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_NEPB, Step::A)?
        };

        // Weighting factors for energy exported to the grid (step A) (~formula 25)
        let f_we_exp_cr_stepA_grid: RenNrenCo2 = if E_exp_cr_grid_an == 0.0 {
            // No energy exported to grid
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_RED, Step::A)?
        };

        // Weighted exported energy according to resources used to generate that energy (formula 23)
        E_we_exp_cr_an_A = (E_exp_cr_used_nEPus_an * f_we_exp_cr_stepA_nEPus) // formula 24
            + (E_exp_cr_grid_an * f_we_exp_cr_stepA_grid); // formula 25

        // * Step B: weighting depends on exported energy generation and avoided resources on the grid

        // Factors of contribution for energy exported to nEP uses (step B)
        let f_we_exp_cr_used_nEPus = if E_exp_cr_used_nEPus_an == 0.0 {
            // No energy exported to nEP uses
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_NEPB, Step::B)?
        };

        // Weighting factors for energy exported to the grid (step B)
        let f_we_exp_cr_grid = if E_exp_cr_grid_an == 0.0 {
            // No energy exported to grid
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_RED, Step::B)?
        };

        // Effect of exported energy on weighted energy performance (step B) (formula 26)

        E_we_exp_cr_used_nEPus_an_AB =
            E_exp_cr_used_nEPus_an * (f_we_exp_cr_used_nEPus - f_we_exp_cr_stepA_nEPus);
        E_we_exp_cr_grid_an_AB = E_exp_cr_grid_an * (f_we_exp_cr_grid - f_we_exp_cr_stepA_grid);
        E_we_exp_cr_an_AB = E_we_exp_cr_used_nEPus_an_AB + E_we_exp_cr_grid_an_AB;

        // Contribution of exported energy to the annual weighted energy performance
        // 11.6.2.1, 11.6.2.2, 11.6.2.3
        E_we_exp_cr_an = E_we_exp_cr_an_A + (k_exp * E_we_exp_cr_an_AB); // (formula 20)
    }

    // * Total result for step A
    // Partial result for carrier (formula 2)
    let E_we_cr_an_A: RenNrenCo2 = E_we_del_cr_an - E_we_exp_cr_an_A;

    // * Total result for step B
    // Partial result for carrier (formula 2)
    let E_we_cr_an: RenNrenCo2 = E_we_del_cr_an - E_we_exp_cr_an;

    // ================ Compute values by use ===============
    // Compute fraction of used energy by use (for EPB services):
    // used energy for service_i / used energy for all services)
    let f_us_cr = compute_factors_by_use_cr(&cr_list);
    // Annual energy use for carrier
    let E_EPus_cr_an = vecsum(&E_EPus_cr_t);

    // Used (final) and Weighted energy for each use item (for EPB services)
    let mut E_Epus_cr_an_byuse: HashMap<Service, f32> = HashMap::new();
    let mut E_we_cr_an_A_byuse: HashMap<Service, RenNrenCo2> = HashMap::new();
    let mut E_we_cr_an_byuse: HashMap<Service, RenNrenCo2> = HashMap::new();
    for service in &SERVICES {
        let f_us_k_cr = *f_us_cr.get(service).unwrap_or(&0.0f32);
        if f_us_k_cr != 0.0 {
            // Used energy
            E_Epus_cr_an_byuse.insert(*service, E_EPus_cr_an * f_us_k_cr);
            // Step A
            E_we_cr_an_A_byuse.insert(*service, E_we_cr_an_A * f_us_k_cr);
            // Step B (E.2.6)
            E_we_cr_an_byuse.insert(*service, E_we_cr_an * f_us_k_cr);
        }
    }

    // === Other aggregated values ===

    // Annual energy use for carrier for non EPB uses
    let E_nEPus_cr_an = vecsum(&E_nEPus_cr_t);
    // Annually produced energy used in EPB uses
    let E_pr_cr_used_EPus_an = vecsum(&E_pr_cr_used_EPus_t);
    // Annually produced energy used in EPB uses by source (INSITU, COGENERACION)
    let E_pr_cr_i_used_EPus_an: HashMap<ProdSource, f32> = E_pr_cr_j_used_EPus_t
        .iter()
        .map(|(source, values)| (*source, vecsum(values)))
        .collect();

    Ok(BalanceForCarrier {
        carrier,
        used_EPB: E_EPus_cr_t,
        used_EPB_an_byuse: E_Epus_cr_an_byuse,
        used_nEPB: E_nEPus_cr_t,
        used_nEPB_an: E_nEPus_cr_an,
        produced: E_pr_cr_t,
        produced_an: E_pr_cr_an,
        produced_by_source: E_pr_cr_j_t,
        produced_by_source_an: E_pr_cr_j_an,
        produced_used_EPus: E_pr_cr_used_EPus_t,
        produced_used_EPus_an: E_pr_cr_used_EPus_an,
        produced_used_EPus_by_source: E_pr_cr_j_used_EPus_t,
        produced_used_EPus_by_source_an: E_pr_cr_i_used_EPus_an,
        f_match: f_match_t,   // load matching factor
        exported: E_exp_cr_t, // exp_used_nEPus + exp_grid
        exported_an: E_exp_cr_an,
        exported_by_source: E_exp_cr_j_t,
        exported_by_source_an: E_exp_cr_j_an,
        exported_grid: E_exp_cr_grid_t,
        exported_grid_an: E_exp_cr_grid_an,
        exported_nEPB: E_exp_cr_used_nEPus_t,
        exported_nEPB_an: E_exp_cr_used_nEPus_an,
        delivered_grid: E_del_cr_t,
        delivered_grid_an: E_del_cr_an,
        // Weighted energy: { ren, nren }
        we_delivered_grid_an: E_we_del_cr_grid_an,
        we_delivered_prod_an: E_we_del_cr_onsite_an,
        we_delivered_an: E_we_del_cr_an,
        we_exported_an_A: E_we_exp_cr_an_A,
        we_exported_nEPB_an_AB: E_we_exp_cr_used_nEPus_an_AB,
        we_exported_grid_an_AB: E_we_exp_cr_grid_an_AB,
        we_exported_an_AB: E_we_exp_cr_an_AB,
        we_exported_an: E_we_exp_cr_an,
        we_an_A: E_we_cr_an_A,
        we_an_A_byuse: E_we_cr_an_A_byuse,
        we_an: E_we_cr_an,
        we_an_byuse: E_we_cr_an_byuse,
    })
}

/// Calcula fracción de cada uso EPB para un vector energético i
///
/// Compute share of each EPB use for a given carrier i
///
/// It uses the reverse calculation method (E.3.6)
/// * `cr_list` - components list for the selected carrier i
///
fn compute_factors_by_use_cr(cr_list: &[EnergyData]) -> HashMap<Service, f32> {
    let mut factors_us_k: HashMap<Service, f32> = HashMap::new();
    // Energy use components (EPB uses) for current carrier i
    let cr_use_list = cr_list.iter().filter(|c| c.is_epb_use());
    // Energy use for all EPB services and carrier i (Q_Epus_cr)
    let q_us_all: f32 = cr_use_list.clone().map(HasValues::values_sum).sum();
    if q_us_all != 0.0 {
        // No energy use for this carrier!
        // Collect share of step A weighted energy for each use item (service)
        for us in SERVICES.iter().cloned() {
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

/// Find weighting factor for 'step' of energy exported to 'dest' from the given energy 'source'.
///
/// * `fp_cr` - weighting factor list for a given energy carrier where search is done
/// * `source` - match this energy source (`RED`, `INSITU`, `COGENERACION`)
/// * `dest` - match this energy destination (use)
/// * `step` - match this calculation step
fn fp_find(
    fp_cr: &[Factor],
    source: impl Into<Source>,
    dest: Dest,
    step: Step,
) -> Result<RenNrenCo2> {
    let source = source.into();
    fp_cr
        .iter()
        .find(|fp| fp.source == source && fp.dest == dest && fp.step == step)
        .map(|fp| fp.factors())
        .ok_or_else(|| {
            EpbdError::MissingFactor(format!(
                "'{}, {}, {}, {}'",
                fp_cr[0].carrier, source, dest, step
            ))
        })
}
