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
Cálculos de balance energético
==============================

Evaluación de la eficiencia energética según la EN ISO 52000-1.

*/

use std::collections::HashMap;

use crate::{
    error::{EpbdError, Result},
    types::{
        Balance, BalanceForCarrier, BalanceTotal, ByServiceEnergy, Carrier, DeliveredEnergy, Dest,
        Energy, ExportedEnergy, HasValues, ProducedEnergy, RenNrenCo2, Service, Source, Step,
        UsedEnergy, WeightedEnergy,
    },
    vecops::{veckmul, vecsum, vecvecdif, vecvecmin, vecvecmul, vecvecsum},
    Components, Factors,
};

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
        // Add up to the global balance
        balance += &bal_cr;
        // Append to the map of balances by carrier
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

    let by_srv = distribute_by_srv(&used, &we, &f_us_cr);

    Ok(BalanceForCarrier {
        carrier,
        f_us: f_us_cr,
        f_match,
        used,
        prod,
        exp,
        del,
        we,
        by_srv,
    })
}

/// Compute used and produced energy data from energy components
///
/// TODO:
/// - Implementar prioridad de consumos entre producciones insitu
/// - Implementar uso de baterías (almacenamiento, sto)
/// - Implementar factor de reparto de carga f_match_t
#[allow(non_snake_case)]
fn compute_used_produced(cr_list: Vec<Energy>) -> (UsedEnergy, ProducedEnergy, Vec<f32>) {
    // We know all carriers have the same time steps (see FromStr for Components)
    let num_steps = cr_list[0].num_steps();

    let mut E_EPus_cr_t = vec![0.0; num_steps];
    let mut E_nEPus_cr_t = vec![0.0; num_steps];
    let mut E_pr_cr_j_t = HashMap::<Source, Vec<f32>>::new();
    for c in &cr_list {
        if c.is_generated() {
            E_pr_cr_j_t
                .entry(c.source())
                .and_modify(|e| *e = vecvecsum(e, c.values()))
                .or_insert_with(|| c.values().to_owned());
        } else if c.is_epb_use() {
            E_EPus_cr_t = vecvecsum(&E_EPus_cr_t, c.values())
        } else {
            // Non EPB use
            E_nEPus_cr_t = vecvecsum(&E_nEPus_cr_t, c.values())
        }
    }
    let E_EPus_cr_an = vecsum(&E_EPus_cr_t);
    let E_nEPus_cr_an = vecsum(&E_nEPus_cr_t);

    // Generation for this carrier from all sources j at each timestep
    let mut E_pr_cr_t = vec![0.0; num_steps];
    // Generation for this carrier from each source for all time steps
    let mut E_pr_cr_j_an = HashMap::<Source, f32>::new();
    for (source, prod_cr_j) in &E_pr_cr_j_t {
        E_pr_cr_t = vecvecsum(&E_pr_cr_t, prod_cr_j);
        E_pr_cr_j_an.insert(*source, vecsum(prod_cr_j));
    }
    let E_pr_cr_an = vecsum(&E_pr_cr_t);

    // Load matching factor with constant value == 1 (11.6.2.4)
    // TODO: implement optional computation of f_match_t (using function in B.32):
    // x = E_pr_cr_t / E_EPus_cr_t (at each time step)
    // f_match_t = if x < 0 { 1.0 } else { (x + 1/x - 1) / (x + 1 / n) };
    let f_match_t = vec![1.0; num_steps];

    let E_pr_cr_used_EPus_t = vecvecmul(&f_match_t, &vecvecmin(&E_EPus_cr_t, &E_pr_cr_t));
    let E_pr_cr_used_EPus_an = vecsum(&E_pr_cr_used_EPus_t);

    // Generated energy from source j used in EP uses
    // Computation without priority (9.6.6.2.4)
    // TODO: implement computation using priorities (9.6.62.4)
    let mut E_pr_cr_j_used_EPus_t = HashMap::<Source, Vec<f32>>::new();
    for (source, prod_cr_j_an) in &E_pr_cr_j_an {
        // * Fraction of produced energy from source j (formula 14)
        // We have grouped by source type (it could be made by generator i, for each one of them)
        let f_pr_cr_j = if E_pr_cr_an > 1e-3 {
            prod_cr_j_an / E_pr_cr_an
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
            epus_t: E_EPus_cr_t,
            epus_an: E_EPus_cr_an,
            nepus_t: E_nEPus_cr_t,
            nepus_an: E_nEPus_cr_an,
        },
        ProducedEnergy {
            t: E_pr_cr_t,
            an: E_pr_cr_an,
            by_src_t: E_pr_cr_j_t,
            by_src_an: E_pr_cr_j_an,
            used_epus_t: E_pr_cr_used_EPus_t,
            used_epus_an: E_pr_cr_used_EPus_an,
            used_epus_by_src_t: E_pr_cr_j_used_EPus_t,
            used_epus_by_src_an: E_pr_cr_j_used_EPus_an,
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
    let E_exp_cr_t = vecvecdif(&prod.t, &prod.used_epus_t);
    let E_exp_cr_used_nEPus_t = vecvecmin(&E_exp_cr_t, &used.nepus_t);
    let E_exp_cr_used_nEPus_an = vecsum(&E_exp_cr_used_nEPus_t);
    let E_exp_cr_grid_t = vecvecdif(&E_exp_cr_t, &E_exp_cr_used_nEPus_t);
    let E_exp_cr_grid_an = vecsum(&E_exp_cr_grid_t);
    let E_del_cr_t = vecvecdif(&used.epus_t, &prod.used_epus_t);
    let E_del_cr_an = vecsum(&E_del_cr_t);
    let E_del_cr_onsite_t = prod
        .by_src_t
        .get(&Source::INSITU)
        .cloned()
        .unwrap_or_else(|| vec![0.0_f32; E_del_cr_t.len()]);
    let E_del_cr_onsite_an = vecsum(&E_del_cr_onsite_t);
    let mut E_exp_cr_j_t = HashMap::<Source, Vec<f32>>::new();
    for (source, prod_src) in &prod.by_src_t {
        E_exp_cr_j_t.insert(*source, vecvecdif(prod_src, &prod.used_epus_by_src_t[source]));
    }
    let mut E_exp_cr_j_an = HashMap::<Source, f32>::new();
    for (source, exp_src) in &E_exp_cr_j_t {
        E_exp_cr_j_an.insert(*source, vecsum(exp_src));
    }
    let E_exp_cr_an = E_exp_cr_used_nEPus_an + E_exp_cr_grid_an;

    (
        ExportedEnergy {
            t: E_exp_cr_t, // exp_used_nEPus + exp_grid
            an: E_exp_cr_an,
            by_src_t: E_exp_cr_j_t,
            by_src_an: E_exp_cr_j_an,
            grid_t: E_exp_cr_grid_t,
            grid_an: E_exp_cr_grid_an,
            used_nepus_t: E_exp_cr_used_nEPus_t,
            used_nepus_an: E_exp_cr_used_nEPus_an,
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
    // Weighted energy for onsite produced energy
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
            for (source, E_exp_cr_gen_an) in &exp.by_src_an {
                result += wfactors.find(carrier, *source, dest, step)? * (E_exp_cr_gen_an / exp.an);
            }
            Ok(result)
        };

        // Weighting factors for energy exported to nEP uses (step A) (~formula 24)
        let f_we_exp_cr_stepA_nEPus: RenNrenCo2 = if exp.used_nepus_an == 0.0 {
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
        E_we_exp_cr_an_A = (exp.used_nepus_an * f_we_exp_cr_stepA_nEPus) // formula 24
            + (exp.grid_an * f_we_exp_cr_stepA_grid); // formula 25

        // * Step B: weighting depends on exported energy generation and avoided resources on the grid

        // Factors of contribution for energy exported to nEP uses (step B)
        let f_we_exp_cr_used_nEPus = if exp.used_nepus_an == 0.0 {
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
            exp.used_nepus_an * (f_we_exp_cr_used_nEPus - f_we_exp_cr_stepA_nEPus);

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

        an_a: E_we_cr_an_A,

        del_an: E_we_del_cr_an,
        del_grid_an: E_we_del_cr_grid_an,
        del_onsite_an: E_we_del_cr_onsite_an,

        exp_an: E_we_exp_cr_an,
        exp_an_a: E_we_exp_cr_an_A,
        exp_an_ab: E_we_exp_cr_an_AB,
        exp_nepus_an_ab: E_we_exp_cr_used_nEPus_an_AB,
        exp_grid_an_ab: E_we_exp_cr_grid_an_AB,
    })
}

/// Distribute used and weighted energy data by EPB service
///
/// Allocate energy to services proportionally to its fraction of used energy over total used energy
#[allow(non_snake_case)]
fn distribute_by_srv(
    used: &UsedEnergy,
    we: &WeightedEnergy,
    f_us_cr: &HashMap<Service, f32>,
) -> ByServiceEnergy {
    let mut used_epus_an: HashMap<Service, f32> = HashMap::new();
    let mut we_an_a: HashMap<Service, RenNrenCo2> = HashMap::new();
    let mut we_an: HashMap<Service, RenNrenCo2> = HashMap::new();
    for service in &Service::SERVICES_EPB {
        let f_us_k_cr = *f_us_cr.get(service).unwrap_or(&0.0f32);
        if f_us_k_cr != 0.0 {
            used_epus_an.insert(*service, used.epus_an * f_us_k_cr);
            we_an_a.insert(*service, we.an_a * f_us_k_cr);
            we_an.insert(*service, we.an * f_us_k_cr);
        }
    }

    ByServiceEnergy {
        used_epus_an,
        we_an_a,
        we_an,
    }
}

/// Calcula fracción de cada uso EPB para un vector energético i
///
/// Compute share of each EPB use for a given carrier i
/// f_us_cr = (used energy for service_i) / (used energy for all services)
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
