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
Cálculos de la eficiencia energética
====================================

Evaluación de la eficiencia energética según la EN ISO 52000-1.

*/

use std::collections::HashMap;

use crate::{
    error::{EpbdError, Result},
    types::{
        Balance, BalanceCarrier, Carrier, DeliveredEnergy, Dest, Energy, EnergyPerformance,
        ExportedEnergy, HasValues, ProdSource, ProducedEnergy, RenNrenCo2, Service, Source, Step,
        UsedEnergy, WeightedEnergy,
    },
    vecops::{vecsum, vecvecdif, vecvecmin, vecvecmul, vecvecsum},
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
/// * `load_matching` - whether statistical load matching is used or not
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
    load_matching: bool,
) -> Result<EnergyPerformance> {
    if arearef < 1e-3 {
        return Err(EpbdError::WrongInput(format!(
            "El área de referencia no puede ser nula o casi nula y se encontró {}",
            arearef
        )));
    };
    let components = components.clone();
    let mut wfactors = wfactors.clone();
    wfactors.add_cgn_factors(&components)?;

    let mut balance = Balance::default();

    // Add energy needs to balance
    for srv in [Service::CAL, Service::REF, Service::ACS] {
        let nd = components
            .building
            .iter()
            .find(|c| c.service == srv)
            .map(HasValues::values_sum);
        if let Some(srv_nd) = nd {
            balance.needs.insert(srv, srv_nd);
        }
    }

    // Compute balance for each carrier and accumulate partial balance values for total balance
    let mut balance_cr: HashMap<Carrier, BalanceCarrier> = HashMap::new();
    for cr in &components.available_carriers() {
        // Compute balance for this carrier ---
        let bal_cr = balance_for_carrier(*cr, &components, &wfactors, k_exp, load_matching)?;
        // Add up to the global balance
        balance += &bal_cr;
        // Append to the map of balances by carrier
        balance_cr.insert(*cr, bal_cr);
    }

    // Compute area weighted total balance
    let balance_m2 = balance.normalize_by_area(arearef);

    // Distant RER
    let rer = balance.we.b.rer();

    // Onsite and nearby RER
    let (rer_onst, rer_nrb) = {
        let tot = balance.we.b.tot();
        if tot > 0.0 {
            let (onst, nrb) = ren_onst_nrb(&balance_cr, k_exp);
            (onst / tot, nrb / tot)
        } else {
            (0.0, 0.0)
        }
    };

    // Energy performance data and results
    Ok(EnergyPerformance {
        components,
        wfactors,
        k_exp,
        arearef,
        balance_cr,
        balance,
        balance_m2,
        rer,
        rer_nrb,
        rer_onst,
        misc: None,
    })
}

/// Renewable energy used (EPB services) from onsite and nearby sources
/// This excludes the impact on the grid of the exported energy
/// Cogen generation is considered onsite (and its renewable contribution depends on the step A factor)
fn ren_onst_nrb(balance_cr: &HashMap<Carrier, BalanceCarrier>, k_exp: f32) -> (f32, f32) {
    // 1. Renewable energy from all nearby carriers (excluding electricity)
    let ren_nrb_cr = balance_cr
        .iter()
        .map(|(carrier, bal)| {
            if carrier.is_nearby() {
                bal.we.b.ren
            } else {
                0.0
            }
        })
        .sum::<f32>();
    let ren_onst_cr = balance_cr
        .iter()
        .map(|(carrier, bal)| {
            if carrier.is_onsite() {
                bal.we.b.ren
            } else {
                0.0
            }
        })
        .sum::<f32>();
    // 2. Renewable energy from onsite produced electricity (excl. cogen)
    let ren_el_onst = balance_cr
        .get(&Carrier::ELECTRICIDAD)
        .map(|cr| cr.we.del_onst.ren)
        .unwrap_or(0.0);
    // 3. Renewable energy from cogeneration
    let ren_el_cgn = balance_cr
        .get(&Carrier::ELECTRICIDAD)
        .map(|cr| cr.we.del_cgn.ren)
        .unwrap_or(0.0);
    // 3. Renewable resources used for exported electricity
    // These have to be substracted depending on k_exp value
    let ren_el_exp_a = balance_cr
        .get(&Carrier::ELECTRICIDAD)
        .map(|cr| cr.we.exp_a.ren)
        .unwrap_or(0.0);
    // 4. Add all contributions
    (
        // Onsite
        ren_onst_cr + ren_el_onst,
        // Nearby
        ren_nrb_cr + ren_el_onst + ren_el_cgn - (1.0 - k_exp) * ren_el_exp_a,
    )
}

// --------------------------------------------------------------------
// Energy calculation functions
// --------------------------------------------------------------------

// ///////////// By Carrier timestep and annual computations ////////////

/// Calcula el balance energético para un vector energético
///
/// Calculate energy balance for a single energy carrier.
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
    load_matching: bool,
) -> Result<BalanceCarrier> {
    let cr_list: Vec<Energy> = components
        .cdata
        .iter()
        .filter(|e| e.has_carrier(carrier))
        .cloned()
        .collect();

    // Compute used and produced energy from components
    let (used, prod, f_match) = compute_used_produced(cr_list, load_matching);

    // Compute exported and delivered energy from used and produced energy data
    let (exp, del) = compute_exported_delivered(&used, &prod);

    let we = compute_weighted_energy(carrier, k_exp, wfactors, &used, &exp, &del)?;

    Ok(BalanceCarrier {
        carrier,
        f_match,
        used,
        prod,
        exp,
        del,
        we,
    })
}

/// Compute used and produced energy data from energy components
///
/// TODO: Battery storage support (sto)
#[allow(non_snake_case)]
fn compute_used_produced(
    cr_list: Vec<Energy>,
    load_matching: bool,
) -> (UsedEnergy, ProducedEnergy, Vec<f32>) {
    // We know all carriers have the same time steps (see FromStr for Components)
    let num_steps = cr_list[0].num_steps();
    let carrier = cr_list[0].carrier();

    let mut E_EPus_cr_t = vec![0.0; num_steps];
    let mut E_EPus_cr_t_by_srv: HashMap<Service, Vec<f32>> = HashMap::new();
    let mut E_nEPus_cr_t = vec![0.0; num_steps];
    let mut E_cgn_in_cr_t = vec![0.0; num_steps];
    let mut E_pr_cr_j_t = HashMap::<ProdSource, Vec<f32>>::new();
    for c in &cr_list {
        let vals = c.values();
        if c.is_generated() {
            // Onsite production + electr. cogeneration
            E_pr_cr_j_t
                .entry(c.prod_source())
                .and_modify(|e| *e = vecvecsum(e, vals))
                .or_insert_with(|| vals.to_owned());
        } else if c.is_epb_use() {
            // EPB services
            E_EPus_cr_t_by_srv
                .entry(c.service())
                .and_modify(|e| *e = vecvecsum(e, vals))
                .or_insert_with(|| vals.to_owned());
            E_EPus_cr_t = vecvecsum(&E_EPus_cr_t, vals)
        } else if c.is_cogen_use() {
            // Cogeneration input
            E_cgn_in_cr_t = vecvecsum(&E_cgn_in_cr_t, vals)
        } else {
            // Non EPB services
            E_nEPus_cr_t = vecvecsum(&E_nEPus_cr_t, vals)
        }
    }
    let E_EPus_cr_an = vecsum(&E_EPus_cr_t);
    let E_nEPus_cr_an = vecsum(&E_nEPus_cr_t);
    let E_cgn_in_cr_an = vecsum(&E_cgn_in_cr_t);

    // Used energy for this carrier for each service for all timesteps
    let mut E_EPus_cr_an_by_srv = HashMap::<Service, f32>::new();
    for (service, epus_srv) in &E_EPus_cr_t_by_srv {
        E_EPus_cr_an_by_srv.insert(*service, vecsum(epus_srv));
    }

    // Generation for this carrier from all sources j at each timestep
    let mut E_pr_cr_t = vec![0.0; num_steps];
    // Generation for this carrier from each source for all time steps
    let mut E_pr_cr_j_an = HashMap::<ProdSource, f32>::new();
    for (source, prod_cr_j) in &E_pr_cr_j_t {
        E_pr_cr_t = vecvecsum(&E_pr_cr_t, prod_cr_j);
        E_pr_cr_j_an.insert(*source, vecsum(prod_cr_j));
    }
    let E_pr_cr_an = vecsum(&E_pr_cr_t);

    // Load matching factor (32) (11.6.2.4)
    let f_match_t = compute_f_match(&E_pr_cr_t, &E_EPus_cr_t, load_matching);

    // Generated energy from source j used in EP
    // If there is more than one source... it could have priorities
    // Compute using priorities priorities (9.6.62.4). EL_INSITU > EL_COGEN
    let (has_priorities, priorities) = ProdSource::get_priorities(carrier);

    let mut E_pr_cr_used_EPus_t = vec![0.0; num_steps];
    let mut E_pr_cr_j_used_EPus_t = HashMap::<ProdSource, Vec<f32>>::new();
    if has_priorities && priorities.iter().all(|s| E_pr_cr_j_an.contains_key(s)) {
        // Energy used for that carrier (9)
        let mut E_EPus_cr_left_t = E_EPus_cr_t.clone();
        // Priorities: sources with a higher priority are used first
        for source in priorities {
            // Max usable production (wrt EP uses) (10)
            let E_pr_cr_j_usmax_t = vecvecmin(&E_pr_cr_j_t[&source], &E_EPus_cr_left_t);
            // Energy left for source with next priority (11)
            E_EPus_cr_left_t = vecvecdif(&E_EPus_cr_left_t, &E_pr_cr_j_usmax_t);
            // Energy used for this priority (12) & add to total used in EPB services
            let used = vecvecmul(&E_pr_cr_j_usmax_t, &f_match_t);
            E_pr_cr_used_EPus_t = vecvecsum(&E_pr_cr_used_EPus_t, &used);
            E_pr_cr_j_used_EPus_t.insert(source, used);
            // Add to total produced and used in EPB services
        }
    } else {
        // No priorities: distribution is proportional to the share of produced energy for each source at each time step
        E_pr_cr_used_EPus_t = vecvecmul(&f_match_t, &vecvecmin(&E_EPus_cr_t, &E_pr_cr_t));
        for (source, prod_cr_j_t) in &E_pr_cr_j_t {
            // * Fraction of produced energy from source j (formula 14)
            // We have grouped by source type (it could be made by generator i, for each one of them)
            let f_pr_cr_j: Vec<_> = prod_cr_j_t
                .iter()
                .zip(E_pr_cr_t.iter())
                .map(|(pr_j, pr_all)| if *pr_all > 1e-3 { pr_j / pr_all } else { 0.0 })
                .collect();
            E_pr_cr_j_used_EPus_t.insert(*source, vecvecmul(&E_pr_cr_used_EPus_t, &f_pr_cr_j));
        }
    }

    let E_pr_cr_used_EPus_an = vecsum(&E_pr_cr_used_EPus_t);

    let E_pr_cr_j_used_EPus_an: HashMap<ProdSource, f32> = E_pr_cr_j_used_EPus_t
        .iter()
        .map(|(source, values)| (*source, vecsum(values)))
        .collect();

    // Compute produced energy used for EPB services by source -----
    // This computes the proportion for each service use for each timestep
    let f_us_cr_by_srv_t = compute_f_us_cr_by_srv_t(&E_EPus_cr_t, &E_EPus_cr_t_by_srv);
    // Along with the produced energy from each source fore each timestep we can distribute produced energy by sources
    let mut E_pr_cr_j_used_EPus_by_srv_by_src_t: HashMap<ProdSource, HashMap<Service, Vec<f32>>> =
        HashMap::new();
    let mut E_pr_cr_j_used_EPus_by_srv_by_src_an: HashMap<ProdSource, HashMap<Service, f32>> =
        HashMap::new();
    for (source, prod) in &E_pr_cr_j_used_EPus_t {
        let mut source_prod_by_srv_t = HashMap::new();
        let mut source_prod_by_srv_an = HashMap::new();
        for (service, factors) in &f_us_cr_by_srv_t {
            let values: Vec<_> = prod
                .iter()
                .zip(factors.iter())
                .map(|(val, f)| f * val)
                .collect();
            let values_an: f32 = values.iter().sum();
            source_prod_by_srv_t.insert(*service, values);
            source_prod_by_srv_an.insert(*service, values_an);
        }
        E_pr_cr_j_used_EPus_by_srv_by_src_t.insert(*source, source_prod_by_srv_t);
        E_pr_cr_j_used_EPus_by_srv_by_src_an.insert(*source, source_prod_by_srv_an);
    }

    (
        UsedEnergy {
            epus_t: E_EPus_cr_t,
            epus_by_srv_t: E_EPus_cr_t_by_srv,
            epus_an: E_EPus_cr_an,
            epus_by_srv_an: E_EPus_cr_an_by_srv,
            nepus_t: E_nEPus_cr_t,
            nepus_an: E_nEPus_cr_an,
            cgnus_t: E_cgn_in_cr_t,
            cgnus_an: E_cgn_in_cr_an,
        },
        ProducedEnergy {
            t: E_pr_cr_t,
            an: E_pr_cr_an,
            by_src_t: E_pr_cr_j_t,
            by_src_an: E_pr_cr_j_an,
            epus_t: E_pr_cr_used_EPus_t,
            epus_an: E_pr_cr_used_EPus_an,
            epus_by_src_t: E_pr_cr_j_used_EPus_t,
            epus_by_src_an: E_pr_cr_j_used_EPus_an,
            epus_by_srv_by_src_t: E_pr_cr_j_used_EPus_by_srv_by_src_t,
            epus_by_srv_by_src_an: E_pr_cr_j_used_EPus_by_srv_by_src_an,
        },
        f_match_t,
    )
}

/// Compute load matching factor (32) (11.6.2.4)
///
/// When load_matching is true it computes the statistical load matching factor using the
/// proposed expression for monthly time steps from table B.32, with k=1 and n=1.
///
/// In other cases, it uses a constant factor = 1.0 for all time steps, as the proposed
/// function for hourly timesteps in table B.32.
#[allow(non_snake_case)]
fn compute_f_match(E_pr_cr_t: &[f32], E_EPus_cr_t: &[f32], load_matching: bool) -> Vec<f32> {
    let num_steps = E_pr_cr_t.len();
    if load_matching {
        // x = E_pr_cr_t / E_EPus_cr_t (at each time step)
        // f_match_t = if x <= 0.0 { 1.0 } else { (x + 1.0/x - 1.0) / (x + 1.0 / x) };
        E_pr_cr_t
            .iter()
            .zip(E_EPus_cr_t.iter())
            .map(|(produced, used)| if *used > 0.0 { produced / used } else { 0.0 })
            .map(|x| {
                if x <= 0.0 {
                    1.0
                } else {
                    (x + 1.0 / x - 1.0) / (x + 1.0 / x)
                }
            })
            .collect()
    } else {
        // Load matching factor with constant value == 1 (11.6.2.4)
        vec![1.0; num_steps]
    }
}

/// Compute exported and delivered energy from used and produced energy data
#[allow(non_snake_case)]
fn compute_exported_delivered(
    used: &UsedEnergy,
    prod: &ProducedEnergy,
) -> (ExportedEnergy, DeliveredEnergy) {
    let E_exp_cr_t = vecvecdif(&prod.t, &prod.epus_t);
    let E_exp_cr_used_nEPus_t = vecvecmin(&E_exp_cr_t, &used.nepus_t);
    let E_exp_cr_used_nEPus_an = vecsum(&E_exp_cr_used_nEPus_t);
    let E_exp_cr_grid_t = vecvecdif(&E_exp_cr_t, &E_exp_cr_used_nEPus_t);
    let E_exp_cr_grid_an = vecsum(&E_exp_cr_grid_t);
    let E_del_cr_t = vecvecdif(&used.epus_t, &prod.epus_t);
    let E_del_cr_an = vecsum(&E_del_cr_t);

    // All energy produced onsite is delivered energy, though part of it can be later exported
    let mut E_del_cr_onsite_t = vec![0.0_f32; E_del_cr_t.len()];
    for (prod_src, prod_values_t) in &prod.by_src_t {
        match (*prod_src).into() {
            Source::INSITU => {
                E_del_cr_onsite_t = vecvecsum(&E_del_cr_onsite_t, prod_values_t);
            }
            _ => continue,
        }
    }
    let E_del_cr_onsite_an = vecsum(&E_del_cr_onsite_t);

    let mut E_exp_cr_j_t = HashMap::<ProdSource, Vec<f32>>::new();
    for (source, prod_src) in &prod.by_src_t {
        E_exp_cr_j_t.insert(*source, vecvecdif(prod_src, &prod.epus_by_src_t[source]));
    }
    let mut E_exp_cr_j_an = HashMap::<ProdSource, f32>::new();
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
            nepus_t: E_exp_cr_used_nEPus_t,
            nepus_an: E_exp_cr_used_nEPus_an,
        },
        DeliveredEnergy {
            an: E_del_cr_an + E_del_cr_onsite_an + used.cgnus_an,
            grid_t: E_del_cr_t,
            grid_an: E_del_cr_an,
            onst_t: E_del_cr_onsite_t,
            onst_an: E_del_cr_onsite_an,
            cgn_t: used.cgnus_t.clone(),
            cgn_an: used.cgnus_an,
        },
    )
}

/// Compute weighted energy from exported and delivered data
#[allow(non_snake_case)]
fn compute_weighted_energy(
    carrier: Carrier,
    k_exp: f32,
    wfactors: &Factors,
    used: &UsedEnergy,
    exp: &ExportedEnergy,
    del: &DeliveredEnergy,
) -> Result<WeightedEnergy> {
    let fP_grid_A = wfactors.find(carrier, Source::RED, Dest::SUMINISTRO, Step::A)?;

    // Weighted energy due to delivered energy from the grid
    let E_we_del_cr_grid_an = del.grid_an * fP_grid_A;

    // Weighted energy due to delivered energy to produce cogenerated electricity
    let E_we_del_cr_cgn_an = if del.cgn_an == 0.0 {
        RenNrenCo2::default()
    } else {
        del.cgn_an * fP_grid_A
    };

    // Weighted energy due to delivered energy from onsite sources
    let E_we_del_cr_onsite_an = if del.onst_an == 0.0 {
        RenNrenCo2::default()
    } else {
        del.onst_an * wfactors.find(carrier, Source::INSITU, Dest::SUMINISTRO, Step::A)?
    };

    let E_we_del_cr_an = E_we_del_cr_grid_an + E_we_del_cr_onsite_an + E_we_del_cr_cgn_an;

    let mut E_we_exp_cr_an = RenNrenCo2::default();
    let mut E_we_exp_cr_an_A = RenNrenCo2::default();
    let mut E_we_exp_cr_nEPus_an_A = RenNrenCo2::default();
    let mut E_we_exp_cr_grid_an_A = RenNrenCo2::default();
    let mut E_we_exp_cr_an_AB = RenNrenCo2::default();
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
                result += wfactors.find(carrier, (*source).into(), dest, step)?
                    * (E_exp_cr_gen_an / exp.an);
            }
            Ok(result)
        };

        // Weighting factors for energy exported to nEP uses (step A) (~formula 24)
        let f_we_exp_cr_stepA_nEPus: RenNrenCo2 = if exp.nepus_an == 0.0 {
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
        E_we_exp_cr_nEPus_an_A = exp.nepus_an * f_we_exp_cr_stepA_nEPus; // formula 24
        E_we_exp_cr_grid_an_A = exp.grid_an * f_we_exp_cr_stepA_grid; // formula 25
        E_we_exp_cr_an_A = E_we_exp_cr_nEPus_an_A + E_we_exp_cr_grid_an_A;

        // * Step B: weighting depends on exported energy generation and avoided resources on the grid

        // Factors of contribution for energy exported to nEP uses (step B)
        // (resources avoided to the grid gen)
        let f_we_exp_cr_used_nEPus = if exp.nepus_an == 0.0 {
            // No energy exported to nEP uses
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_NEPB, Step::B)?
        };

        // Weighting factors for energy exported to the grid (step B)
        // (resources avoided to the grid gen)
        let f_we_exp_cr_grid = if exp.grid_an == 0.0 {
            // No energy exported to grid
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            f_we_exp_cr_compute(Dest::A_RED, Step::B)?
        };

        // Effect of exported energy on weighted energy performance (step B) (formula 26)

        E_we_exp_cr_used_nEPus_an_AB =
            exp.nepus_an * (f_we_exp_cr_used_nEPus - f_we_exp_cr_stepA_nEPus); // formula 27

        E_we_exp_cr_grid_an_AB = exp.grid_an * (f_we_exp_cr_grid - f_we_exp_cr_stepA_grid); // formula 28

        E_we_exp_cr_an_AB = E_we_exp_cr_used_nEPus_an_AB + E_we_exp_cr_grid_an_AB; // formula 26

        // Contribution of exported energy to the annual weighted energy performance
        // 11.6.2.1, 11.6.2.2, 11.6.2.3
        E_we_exp_cr_an = E_we_exp_cr_an_A + (k_exp * E_we_exp_cr_an_AB); // (formula 20)
    }
    let E_we_cr_an_A: RenNrenCo2 = E_we_del_cr_an - E_we_exp_cr_an_A;
    let E_we_cr_an: RenNrenCo2 = E_we_del_cr_an - E_we_exp_cr_an;

    // Compute fraction of used energy for each EPB service:
    // f_us_cr = (used energy for service_i) / (used energy for all services)
    // This uses the reverse calculation method (E.3.6)
    let f_us_cr = compute_f_us_cr_an(used);
    let mut E_we_cr_an_A_by_srv: HashMap<Service, RenNrenCo2> = HashMap::new();
    let mut E_we_cr_an_by_srv: HashMap<Service, RenNrenCo2> = HashMap::new();
    for (service, f_us_k_cr) in f_us_cr {
        E_we_cr_an_A_by_srv.insert(service, E_we_cr_an_A * f_us_k_cr);
        E_we_cr_an_by_srv.insert(service, E_we_cr_an * f_us_k_cr);
    }

    Ok(WeightedEnergy {
        b: E_we_cr_an,
        b_by_srv: E_we_cr_an_by_srv,
        a: E_we_cr_an_A,
        a_by_srv: E_we_cr_an_A_by_srv,

        del: E_we_del_cr_an,
        del_grid: E_we_del_cr_grid_an,
        del_onst: E_we_del_cr_onsite_an,
        del_cgn: E_we_del_cr_cgn_an,

        exp: E_we_exp_cr_an,
        exp_a: E_we_exp_cr_an_A,
        exp_nepus_a: E_we_exp_cr_nEPus_an_A,
        exp_grid_a: E_we_exp_cr_grid_an_A,
        exp_ab: E_we_exp_cr_an_AB,
        exp_nepus_ab: E_we_exp_cr_used_nEPus_an_AB,
        exp_grid_ab: E_we_exp_cr_grid_an_AB,
    })
}

/// Calcula fracción de cada uso EPB para un vector energético i
///
/// Compute share of each EPB use for a given carrier i
/// f_us_cr = (used energy for service_i) / (used energy for all services)
///
/// It uses the reverse calculation method (E.3.6)
/// * `cr_list` - components list for the selected carrier i
///
fn compute_f_us_cr_an(used: &UsedEnergy) -> HashMap<Service, f32> {
    let mut factors_us_k: HashMap<Service, f32> = HashMap::new();

    for (service, used_srv) in &used.epus_by_srv_an {
        let f = if used.epus_an > 0.0 {
            used_srv / used.epus_an
        } else {
            0.0
        };
        factors_us_k.insert(*service, f);
    }
    factors_us_k
}

/// Calcula fracción de cada uso EPB para un vector energético i para cada paso de cálculo
///
/// Compute share of each EPB use for a given carrier i
/// f_us_cr = (used energy for service_i) / (used energy for all services)
///
/// It uses the reverse calculation method (E.3.6)
/// * `cr_list` - components list for the selected carrier i
///
fn compute_f_us_cr_by_srv_t(
    epus_t: &[f32],
    epus_by_srv_t: &HashMap<Service, Vec<f32>>,
) -> HashMap<Service, Vec<f32>> {
    let mut factors_us_k: HashMap<Service, Vec<f32>> = HashMap::new();

    for (service, used_srv) in epus_by_srv_t {
        let f = used_srv
            .iter()
            .zip(epus_t.iter())
            .map(|(used_srv_t, used_t)| {
                if *used_t > 0.0 {
                    used_srv_t / used_t
                } else {
                    0.0
                }
            })
            .collect();
        factors_us_k.insert(*service, f);
    }
    factors_us_k
}
