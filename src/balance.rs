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
Tipos para el balance energético
================================

Definición de los tipos Balance, BalanceForCarrier and BalanceTotal
y de los métodos que implementan la evaluación de la eficiencia energética
según la EN ISO 52000-1.

*/

use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use crate::{
    error::{EpbdError, Result},
    types::{
        CSubtype, CType, Carrier, Component, Dest, Factor, RenNrenCo2, Service, Source, Step,
        SERVICES,
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

    let carriers: HashSet<_> = components.cdata.iter().map(|e| e.carrier).collect();

    // Compute balance for each carrier
    let mut balance_cr: HashMap<Carrier, BalanceForCarrier> = HashMap::new();
    for &carrier in &carriers {
        let components_cr: Vec<Component> = components
            .cdata
            .iter()
            .filter(|e| e.carrier == carrier)
            .cloned()
            .collect();
        let fp_cr: Vec<Factor> = wfactors
            .wdata
            .iter()
            .filter(|e| e.carrier == carrier)
            .cloned()
            .collect();
        let bal = balance_for_carrier(carrier, &components_cr, &fp_cr, k_exp)?;
        balance_cr.insert(carrier, bal);
    }

    // Accumulate partial balance values for total balance
    let balance: BalanceTotal = carriers
        .iter()
        .fold(BalanceTotal::default(), |mut acc, cr| {
            // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step A
            acc.A += balance_cr[cr].we_an_A;
            // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step B
            acc.B += balance_cr[cr].we_an;
            // Weighted energy partials
            acc.we_del += balance_cr[cr].we_delivered_an;
            acc.we_exp_A += balance_cr[cr].we_exported_an_A;
            acc.we_exp += balance_cr[cr].we_exported_an;
            // Weighted energy for each use item (EPB services)
            for &service in &SERVICES {
                // Energy use
                if let Some(value) = balance_cr[cr].used_EPB_an_byuse.get(&service) {
                    *acc.used_EPB_byuse.entry(service).or_default() += *value
                }
                // Step A
                if let Some(value) = balance_cr[cr].we_an_A_byuse.get(&service) {
                    *acc.A_byuse.entry(service).or_default() += *value
                }
                // Step B
                if let Some(value) = balance_cr[cr].we_an_byuse.get(&service) {
                    *acc.B_byuse.entry(service).or_default() += *value;
                }
            }
            acc
        });

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
/// * Missing weighting factors for a carrier, origin, destination or calculation step
///
#[allow(non_snake_case)]
fn balance_for_carrier(
    carrier: Carrier,
    cr_list: &[Component],
    fp_cr: &[Factor],
    k_exp: f32,
) -> Result<BalanceForCarrier> {
    // We know all carriers have the same timesteps (see FromStr for Components)
    let num_steps = cr_list[0].values.len();

    // * Energy used by technical systems for EPB services, for each time step
    let E_EPus_cr_t = cr_list
        .iter()
        .filter(|e| e.ctype == CType::CONSUMO && e.csubtype == CSubtype::EPB)
        .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, &e.values));

    // * Energy used by technical systems for non-EPB services, for each time step
    let E_nEPus_cr_t = cr_list
        .iter()
        .filter(|e| e.ctype == CType::CONSUMO && e.csubtype == CSubtype::NEPB)
        .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, &e.values));

    // * Produced on-site energy and inside the assessment boundary, by generator i (origin i)
    let mut E_pr_cr_i_t = HashMap::<CSubtype, Vec<f32>>::new();
    for comp in cr_list
        .iter()
        .filter(|comp| comp.ctype == CType::PRODUCCION)
    {
        E_pr_cr_i_t
            .entry(comp.csubtype)
            .and_modify(|e| *e = vecvecsum(e, &comp.values))
            .or_insert_with(|| comp.values.clone());
    }

    // PRODUCED ENERGY GENERATORS (CSubtype::INSITU or CSubtype::COGENERACION)
    // generators are unique in this list
    let pr_generators: Vec<CSubtype> = E_pr_cr_i_t.keys().cloned().collect(); // INSITU, COGENERACION

    // Annually produced on-site energy from generator i (origin i)
    let mut E_pr_cr_i_an = HashMap::<CSubtype, f32>::new();
    for gen in &pr_generators {
        E_pr_cr_i_an.insert(*gen, vecsum(&E_pr_cr_i_t[gen]));
    }

    // * Energy produced on-site and inside the assessment boundary (formula 30)
    let mut E_pr_cr_t = vec![0.0; num_steps];
    for gen in &pr_generators {
        E_pr_cr_t = vecvecsum(&E_pr_cr_t, &E_pr_cr_i_t[gen])
    }
    let E_pr_cr_an = vecsum(&E_pr_cr_t);

    // * Produced energy from all origins for EPB services for each time step (formula 31)
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

    // Exported energy by generator i (origin) (9.6.6.2)
    // Implementation WITHOUT priorities on energy use

    // * Fraction of produced energy of type i (origin from generator i) (formula 14)
    let mut f_pr_cr_i = HashMap::<CSubtype, f32>::new();
    for gen in &pr_generators {
        let f = if E_pr_cr_an > 1e-3 {
            E_pr_cr_i_an[gen] / E_pr_cr_an
        } else {
            0.0
        };
        f_pr_cr_i.insert(*gen, f);
    }

    // * Produced energy with origin from generator i and used for EPB services (formula 15)
    let mut E_pr_cr_i_used_EPus_t = HashMap::<CSubtype, Vec<f32>>::new();
    for gen in &pr_generators {
        E_pr_cr_i_used_EPus_t.insert(*gen, veckmul(&E_pr_cr_used_EPus_t, f_pr_cr_i[gen]));
    }

    // * Exported energy from generator i (origin i) (formula 16)
    let mut E_exp_cr_i_t = HashMap::<CSubtype, Vec<f32>>::new();
    for gen in &pr_generators {
        E_exp_cr_i_t.insert(
            *gen,
            vecvecdif(&E_pr_cr_i_t[gen], &E_pr_cr_i_used_EPus_t[gen]),
        );
    }

    // * Annually exported energy from generator i (origin i)
    let mut E_exp_cr_i_an = HashMap::<CSubtype, f32>::new();
    for gen in &pr_generators {
        E_exp_cr_i_an.insert(*gen, vecsum(&E_exp_cr_i_t[gen]));
    }

    // -------- Weighted delivered and exported energy (11.6.2.1, 11.6.2.2, 11.6.2.3 + eq 2, 3)
    // NOTE: All weighting factors have been considered constant through all timesteps
    // NOTE: This allows using annual quantities and not timestep expressions

    // Find weighting factor for 'step' of energy exported to 'dest' from the given energy 'source'.
    //
    // * `fp_cr` - weighting factor list for a given energy carrier where search is done
    // * `source` - match this energy source (`RED`, `INSITU`, `COGENERACION`)
    // * `dest` - match this energy destination (use)
    // * `step` - match this calculation step
    fn fp_find(fp_cr: &[Factor], source: Source, dest: Dest, step: Step) -> Result<&Factor> {
        fp_cr
            .iter()
            .find(|fp| fp.source == source && fp.dest == dest && fp.step == step)
            .ok_or_else(|| {
                EpbdError::MissingFactor(format!(
                    "'{}, {}, {}, {}'",
                    fp_cr[0].carrier, source, dest, step
                ))
            })
    }

    // * Weighted energy for delivered energy: the cost of producing that energy
    let fpA_grid = fp_find(fp_cr, Source::RED, Dest::SUMINISTRO, Step::A)?;
    let E_we_del_cr_grid_an = E_del_cr_an * fpA_grid.factors(); // formula 19, 39

    // 2) Delivered energy from non cogeneration on-site sources (origin i)
    let E_we_del_cr_onsite_an = E_pr_cr_i_an
        .get(&CSubtype::INSITU)
        .and_then(|E_pr_cr_i| {
            fp_find(fp_cr, Source::INSITU, Dest::SUMINISTRO, Step::A)
                .and_then(|fpA_pr_cr_i| Ok(E_pr_cr_i * fpA_pr_cr_i.factors()))
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
        // or becuause there's no effective exportation
        // * Step A: weighting depends on exported energy generation (origin generator)
        // Factors are averaged weighting by production for each origin (no priority, 9.6.6.2.4)

        // * Fraction of produced energy tipe i (origin from generator i) that is exported (formula 14)
        // NOTE: simplified for annual computations (not valid for timestep calculation)
        let mut f_pr_cr_i = HashMap::<CSubtype, f32>::new();
        for gen in &pr_generators {
            // Do not store generators without generation
            if E_exp_cr_i_an[gen] != 0.0 {
                f_pr_cr_i.insert(*gen, vecsum(&E_exp_cr_i_t[gen]) / E_exp_cr_i_an[gen]);
            }
        }
        // Generators (produced energy sources) that are exporting some energy (!= 0)
        let exp_generators: Vec<_> = f_pr_cr_i.keys().collect();

        // Weighting factors for energy exported to nEP uses (step A) (~formula 24)
        let f_we_exp_cr_stepA_nEPus: RenNrenCo2 = if E_exp_cr_used_nEPus_an == 0.0 {
            // No exported energy to nEP uses
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            exp_generators.iter().fold(
                Ok(RenNrenCo2::default()),
                |acc: Result<RenNrenCo2>, &gen| {
                    let fp = fp_find(fp_cr, (*gen).try_into()?, Dest::A_NEPB, Step::A)?;
                    Ok(acc? + (fp.factors() * f_pr_cr_i[gen]))
                },
            )? // sum all i (non grid sources): fpA_nEPus_i[gen] * f_pr_cr_i[gen]
        };

        // Weighting factors for energy exported to the grid (step A) (~formula 25)
        let f_we_exp_cr_stepA_grid: RenNrenCo2 = if E_exp_cr_grid_an == 0.0 {
            // No energy exported to grid
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            exp_generators.iter().fold(
                Ok(RenNrenCo2::default()),
                |acc: Result<RenNrenCo2>, &gen| {
                    let fp = fp_find(fp_cr, (*gen).try_into()?, Dest::A_RED, Step::A)?;
                    Ok(acc? + (fp.factors() * f_pr_cr_i[gen]))
                },
            )? // sum all i (non grid sources): fpA_grid_i[gen] * f_pr_cr_i[gen];
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
            exp_generators.iter().fold(
                Ok(RenNrenCo2::default()),
                |acc: Result<RenNrenCo2>, &gen| {
                    let fp = fp_find(fp_cr, (*gen).try_into()?, Dest::A_NEPB, Step::B)?;
                    Ok(acc? + (fp.factors() * f_pr_cr_i[gen]))
                },
            )? // sum all i (non grid sources): fpB_nEPus_i[gen] * f_pr_cr_i[gen]
        };

        // Weighting factors for energy exported to the grid (step B)
        let f_we_exp_cr_grid = if E_exp_cr_grid_an == 0.0 {
            // No energy exported to grid
            RenNrenCo2::default() // ren: 0.0, nren: 0.0, co2: 0.0
        } else {
            exp_generators.iter().fold(
                Ok(RenNrenCo2::default()),
                |acc: Result<RenNrenCo2>, &gen| {
                    let fp = fp_find(fp_cr, (*gen).try_into()?, Dest::A_RED, Step::B)?;
                    Ok(acc? + (fp.factors() * f_pr_cr_i[gen]))
                },
            )? // sum all i (non grid sources): fpB_grid_i[gen] * f_pr_cr_i[gen];
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
    let f_us_cr = compute_factors_by_use_cr(cr_list);
    // Annual energy use for carrier
    let E_EPus_cr_an: f32 = E_EPus_cr_t.iter().sum();

    // Used (final) and Weighted energy for each use item (for EPB services)
    let mut E_Epus_cr_an_byuse: HashMap<Service, f32> = HashMap::new();
    let mut E_we_cr_an_A_byuse: HashMap<Service, RenNrenCo2> = HashMap::new();
    let mut E_we_cr_an_byuse: HashMap<Service, RenNrenCo2> = HashMap::new();
    for service in &SERVICES {
        let f_us_k_cr = *f_us_cr.get(service).unwrap_or(&0.0f32);
        if f_us_k_cr != 0.0 {
            // Used energy
            E_Epus_cr_an_byuse.insert(service.clone(), E_EPus_cr_an * f_us_k_cr);
            // Step A
            E_we_cr_an_A_byuse.insert(service.clone(), E_we_cr_an_A * f_us_k_cr);
            // Step B (E.2.6)
            E_we_cr_an_byuse.insert(service.clone(), E_we_cr_an * f_us_k_cr);
        }
    }

    Ok(BalanceForCarrier {
        carrier,
        used_EPB: E_EPus_cr_t,
        used_EPB_an_byuse: E_Epus_cr_an_byuse,
        used_nEPB: E_nEPus_cr_t,
        produced: E_pr_cr_t,
        produced_an: E_pr_cr_an,
        produced_bygen: E_pr_cr_i_t,
        produced_bygen_an: E_pr_cr_i_an,
        produced_used_EPus: E_pr_cr_used_EPus_t,
        produced_used_EPus_bygen: E_pr_cr_i_used_EPus_t,
        f_match: f_match_t,   // load matching factor
        exported: E_exp_cr_t, // exp_used_nEPus + exp_grid
        exported_an: E_exp_cr_an,
        exported_bygen: E_exp_cr_i_t,
        exported_bygen_an: E_exp_cr_i_an,
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
fn compute_factors_by_use_cr(cr_list: &[Component]) -> HashMap<Service, f32> {
    let mut factors_us_k: HashMap<Service, f32> = HashMap::new();
    // Energy use components (EPB uses) for current carrier i
    let cr_use_list = cr_list
        .iter()
        .filter(|c| c.ctype == CType::CONSUMO && c.csubtype == CSubtype::EPB);
    // Energy use for all EPB services and carrier i (Q_Epus_cr)
    let q_us_all: f32 = cr_use_list
        .clone()
        .map(|c| c.values.iter().sum::<f32>())
        .sum();
    if q_us_all != 0.0 {
        // No energy use for this carrier!
        // Collect share of step A weighted energy for each use item (service)
        for us in SERVICES.iter().cloned() {
            // Energy use for use k
            let q_us_k: f32 = cr_use_list
                .clone()
                .filter(|c| c.service == us)
                .map(|c| c.values.iter().sum::<f32>())
                .sum();
            // Factor for use k
            factors_us_k.insert(us, q_us_k / q_us_all);
        }
    }
    factors_us_k
}
