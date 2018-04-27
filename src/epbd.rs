// Copyright (c) 2018 Ministerio de Fomento
//                    Instituto de Ciencias de la Construcción Eduardo Torroja (IETcc-CSIC)

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
//            Daniel Jiménez González <dani@ietcc.csic.es>

// ENERGYCALCULATIONS - Implementation of the ISO EN 52000-1 standard

//   Energy performance of buildings - Overarching EPB assessment - General framework and procedures

//   This implementation has used the following assumptions:
//   - weighting factors are constant for all timesteps
//   - no priority is set for energy production (average step A weighting factor f_we_el_stepA)
//   - all on-site produced energy from non cogeneration sources is considered as delivered
//   - the load matching factor is constant and equal to 1.0

//   TODO:
//   - allow other values of the load matching factor (or functions) (formula 32, B.32)
//   - get results by use items (service), maybe using the reverse method E.3 (E.3.6, E.3.7)

use std::collections::HashMap;

use failure::Error;
//use failure::ResultExt;

use types::{CSubtype, CType, Carrier, Dest, Source, Step};
use types::{RenNren, CarrierBalance, TBalance, TComponent, TComponents, TFactor, TFactors, TTotalBalance};

use vecops::{veckmul, veclistsum, vecsum, vecvecdif, vecvecmin, vecvecmul, vecvecsum};

// --------------------------------------------------------------------
// Energy calculation functions
// --------------------------------------------------------------------

// ///////////// By Carrier timestep and annual computations ////////////

// Calculate energy balance for carrier
//
//    cr_i_list: list of components for carrier_i
//    k_exp: exported energy factor [0, 1]
//    fp_cr: weighting factors for carrier
//
//    This follows the ISO EN 52000-1 procedure for calculation of delivered,
//    exported and weighted energy balance.
//
// TODO: Move filtering and selection of weighting factors to a fn to simplify code
pub fn balance_cr(
    cr_i_list: &[TComponent],
    fp_cr: &[TFactor],
    k_exp: f32,
) -> Result<CarrierBalance, Error> {
    let CURRENTCARRIER = &cr_i_list[0].carrier;
    let num_steps = cr_i_list[0].values.len(); // All carriers have the same timesteps (see FromStr for TComponents)

    // * Energy used by technical systems for EPB services, for each time step
    let E_EPus_cr_t = cr_i_list
        .iter()
        .filter(|e| e.ctype == CType::CONSUMO)
        .filter(|e| e.csubtype == CSubtype::EPB)
        .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, &e.values));

    // * Energy used by technical systems for non-EPB services, for each time step
    let E_nEPus_cr_t = cr_i_list
        .iter()
        .filter(|e| e.ctype == CType::CONSUMO)
        .filter(|e| e.csubtype == CSubtype::NEPB)
        .fold(vec![0.0; num_steps], |acc, e| vecvecsum(&acc, &e.values));

    // * Produced on-site energy and inside the assessment boundary, by generator i (origin i)
    let acc_hash: HashMap<CSubtype, Vec<f32>> = HashMap::new();
    let E_pr_cr_pr_i_t = cr_i_list
        .iter()
        .filter(|cr| cr.ctype == CType::PRODUCCION)
        .fold(acc_hash, |mut acc, cr| {
            {
                let vals = acc.entry(cr.csubtype)
                    .or_insert_with(|| vec![0.0; num_steps]);
                *vals = vecvecsum(vals, &cr.values);
            }
            acc
        });

    // PRODUCED ENERGY GENERATORS (CSubtype::INSITU or CSubtype::COGENERACION)
    let pr_generators: Vec<CSubtype> = E_pr_cr_pr_i_t.keys().cloned().collect(); // INSITU, COGENERACION

    // Annually produced on-site energy from generator i (origin i)
    let acc_hash: HashMap<CSubtype, f32> = HashMap::new();
    let E_pr_cr_pr_i_an = pr_generators.iter().fold(acc_hash, |mut acc, gen| {
        {
            let val = acc.entry(*gen).or_insert(0.0);
            *val += vecsum(&E_pr_cr_pr_i_t[gen]);
        }
        acc
    });

    // * Energy produced on-site and inside the assessment boundary (formula 30)
    let E_pr_cr_t = if !pr_generators.is_empty() {
        let pr_vals: Vec<_> = pr_generators
            .iter()
            .map(|gen| &E_pr_cr_pr_i_t[gen])
            .collect();
        veclistsum(pr_vals.as_slice())
    } else {
        vec![0.0; num_steps]
    };
    let E_pr_cr_an = vecsum(&E_pr_cr_t);

    // * Produced energy from all origins for EPB services for each time step (formula 31)
    // TODO: f_match_t constant for electricity (formula 32)
    // TODO: let f_match_t = fmatch(E_pr_cr_t / E_EPus_cr_t)
    let f_match_t = vec![1.0; E_EPus_cr_t.len()];

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
    let acc_hash: HashMap<CSubtype, f32> = HashMap::new();
    let f_pr_cr_i = pr_generators.iter().fold(acc_hash, |mut acc, gen| {
        {
            let val = acc.entry(*gen).or_insert(0.0);
            if E_pr_cr_an > 1e-3 {
                *val = E_pr_cr_pr_i_an[gen] / E_pr_cr_an;
            }
        }
        acc
    });

    // * Energy used for produced carrier energy type i (origin from generator i) (formula 15)
    let acc_hash: HashMap<CSubtype, Vec<f32>> = HashMap::new();
    let E_pr_cr_i_used_EPus_t = pr_generators.iter().fold(acc_hash, |mut acc, gen| {
        {
            let val = acc.entry(*gen).or_insert_with(|| vec![0.0; num_steps]);
            *val = veckmul(&E_pr_cr_used_EPus_t, f_pr_cr_i[gen]);
        }
        acc
    });

    // * Exported energy from generator i (origin i) (formula 16)
    let acc_hash: HashMap<CSubtype, Vec<f32>> = HashMap::new();
    let E_exp_cr_pr_i_t = pr_generators.iter().fold(acc_hash, |mut acc, gen| {
        {
            let val = acc.entry(*gen).or_insert_with(|| vec![0.0; num_steps]);
            *val = vecvecdif(&E_pr_cr_pr_i_t[gen], &E_pr_cr_i_used_EPus_t[gen]);
        }
        acc
    });

    // * Annually exported energy from generator i (origin i)
    let acc_hash: HashMap<CSubtype, f32> = HashMap::new();
    let E_exp_cr_pr_i_an = pr_generators.iter().fold(acc_hash, |mut acc, gen| {
        {
            let val = acc.entry(*gen).or_insert(0.0);
            *val = vecsum(&E_exp_cr_pr_i_t[gen]);
        }
        acc
    });

    // -------- Weighted delivered and exported energy (11.6.2.1, 11.6.2.2, 11.6.2.3 + eq 2, 3)
    // NOTE: All weighting factors have been considered constant through all timesteps
    // NOTE: This allows using annual quantities and not timestep expressions

    // * Weighted energy for delivered energy: the cost of producing that energy
    // TODO: we could avoid forcing the definition of some factors if no related energy input does exist

    // 1) Delivered energy from the grid
    // NOTE: grid delivered energy is energy which is used but not produced (on-site or nearby)
    let fpA_grid: &TFactor = fp_cr
        .iter()
        .find(|fp| fp.dest == Dest::input && fp.step == Step::A && fp.source == Source::RED)
        .ok_or_else(|| {
            format_err!(
                "No weighting factor found for: '{}, RED, input, A'",
                CURRENTCARRIER
            )
        })?;
    let E_we_del_cr_grid_an = E_del_cr_an * fpA_grid.factors(); // formula 19, 39

    // 2) Delivered energy from non cogeneration on-site sources
    let fpA_pr_i: &TFactor = fp_cr
        .iter()
        .find(|fp| fp.dest == Dest::input && fp.step == Step::A && fp.source == Source::INSITU)
        .ok_or_else(|| {
            format_err!(
                "No weighting factor found for: '{}, INSITU, input, A'",
                CURRENTCARRIER
            )
        })?;

    let E_we_del_cr_pr_an = {
        let E_pr_i = E_pr_cr_pr_i_an.get(&CSubtype::INSITU).unwrap_or(&0.0);
        E_pr_i * fpA_pr_i.factors()
    };

    // 3) Total delivered energy: grid + all non cogeneration
    let E_we_del_cr_an = E_we_del_cr_grid_an + E_we_del_cr_pr_an; // formula 19, 39

    // // * Weighted energy for exported energy: depends on step A or B

    let mut E_we_exp_cr_an_A = RenNren::new();
    let mut E_we_exp_cr_an_AB = RenNren::new();
    let mut E_we_exp_cr_an = RenNren::new();
    let mut E_we_exp_cr_used_nEPus_an_AB = RenNren::new();
    let mut E_we_exp_cr_grid_an_AB = RenNren::new();

    let E_exp_cr_an = E_exp_cr_used_nEPus_an + E_exp_cr_grid_an;

    if E_exp_cr_an != 0.0 {
        // This case implies there is exported energy.
        // If there's no exportation, it's either because the carrier cannot be exported
        // or becuause there's no effective exportation
        // * Step A: weighting depends on exported energy generation (origin generator)
        // Factors are averaged weighting by production for each origin (no priority, 9.6.6.2.4)

        // * Fraction of produced energy tipe i (origin from generator i) that is exported (formula 14)
        // NOTE: simplified for annual computations (not valid for timestep calculation)
        let acc_hash: HashMap<CSubtype, f32> = HashMap::new();
        let F_pr_i = pr_generators.iter().fold(acc_hash, |mut acc, gen| {
            let E_exp_cr_gen_an = E_exp_cr_pr_i_an[gen];
            if E_exp_cr_gen_an == 0.0 {
                return acc;
            } // Don't store generators without generation
            acc.insert(*gen, vecsum(&E_exp_cr_pr_i_t[gen]) / E_exp_cr_gen_an);
            acc
        });
        // Generators (produced energy sources) that are exporting some energy (!= 0)
        let exp_generators: Vec<_> = F_pr_i.keys().collect();

        // Weighting factors for energy exported to nEP uses (step A) (~formula 24)
        let f_we_exp_cr_stepA_nEPus: RenNren = if E_exp_cr_used_nEPus_an == 0.0 {
            RenNren::new() // ren: 0.0, nren: 0.0
        } else {
            // There's exported energy to nEP uses
            let mut fpA_nEPus_i = fp_cr
                .iter()
                .filter(|fp| fp.dest == Dest::to_nEPB && fp.step == Step::A);
            // TODO: A check to ensure that fpA_g exist for all gen values should move here from the closure
            exp_generators
                .iter()
                .fold(Ok(RenNren::new()), |acc: Result<RenNren, Error>, &gen| {
                    let acc_ok = acc?;
                    let fpA_nEPus_i_gen = fpA_nEPus_i
                        .find(|fp| {
                            // NOTE: Convert from CSubtype (TCOMPONENT type) to Source (WFACTOR)
                            match *gen {
                                CSubtype::INSITU => fp.source == Source::INSITU,
                                CSubtype::COGENERACION => fp.source == Source::COGENERACION,
                                _ => false,
                            }
                        })
                        .ok_or_else(|| {
                            format_err!(
                                "No weighting factor found for: '{}, {}, to_nEPB, A'",
                                CURRENTCARRIER,
                                gen
                            )
                        })?;
                    Ok(acc_ok + (fpA_nEPus_i_gen.factors() * F_pr_i[gen]))
                })? // suma de todos los i: fpA_nEPus_i * F_pr_i[gen]
        };

        // Weighting factors for energy exported to the grid (step A) (~formula 25)
        let f_we_exp_cr_stepA_grid: RenNren = if E_exp_cr_grid_an == 0.0 {
            // No energy exported to grid
            RenNren::new() // ren: 0.0, nren: 0.0
        } else {
            let mut fpA_grid_i = fp_cr
                .iter()
                .filter(|fp| fp.dest == Dest::to_grid && fp.step == Step::A);
            // TODO: A check to ensure that fpA_g exist for all gen values should move here from the closure
            exp_generators
                .iter()
                .fold(Ok(RenNren::new()), |acc: Result<RenNren, Error>, &gen| {
                    let acc_ok = acc?;
                    let fpA_grid_i_gen = fpA_grid_i
                        .find(|fp| {
                            // NOTE: Convert from CSubtype (TCOMPONENT type) to Source (WFACTOR)
                            match *gen {
                                CSubtype::INSITU => fp.source == Source::INSITU,
                                CSubtype::COGENERACION => fp.source == Source::COGENERACION,
                                _ => false,
                            }
                        })
                        .ok_or_else(|| {
                            format_err!(
                                "No weighting factor found for: '{}, {}, to_nEPB, A'",
                                CURRENTCARRIER,
                                gen
                            )
                        })?;
                    Ok(acc_ok + (fpA_grid_i_gen.factors() * F_pr_i[gen]))
                })? // suma de todos los i: fpA_grid_i * F_pr_i[gen];
        };

        // Weighted exported energy according to resources used to generate that energy (formula 23)
        E_we_exp_cr_an_A = (E_exp_cr_used_nEPus_an * f_we_exp_cr_stepA_nEPus) // formula 24
            + (E_exp_cr_grid_an * f_we_exp_cr_stepA_grid); // formula 25

        // * Step B: weighting depends on exported energy generation and avoided resources on the grid

        // Factors of contribution for energy exported to nEP uses (step B)
        let f_we_exp_cr_used_nEPus = if E_exp_cr_used_nEPus_an == 0.0 {
            // No energy exported to nEP uses
            RenNren::new() // ren: 0.0, nren: 0.0
        } else {
            let mut fpB_nEPus_i = fp_cr
                .iter()
                .filter(|fp| fp.dest == Dest::to_nEPB && fp.step == Step::B);
            exp_generators
                .iter()
                .fold(Ok(RenNren::new()), |acc: Result<RenNren, Error>, &gen| {
                    let acc_ok = acc?;
                    let fpB_nEPus_i_gen = fpB_nEPus_i
                        .find(|fp|
                        // NOTE: Convert from CSubtype (TCOMPONENT type) to Source (WFACTOR)
                        match *gen {
                            CSubtype::INSITU => fp.source == Source::INSITU,
                            CSubtype::COGENERACION => fp.source == Source::COGENERACION,
                            _ => false,
                        })
                        .ok_or_else(|| {
                            format_err!(
                                "No weighting factor found for: '{}, {}, to_nEPB, B'",
                                CURRENTCARRIER,
                                gen
                            )
                        })?;
                    Ok(acc_ok + (fpB_nEPus_i_gen.factors() * F_pr_i[gen]))
                })? // suma de todos los i: fpB_nEPus_i * F_pr_i[gen]
        };

        // Weighting factors for energy exported to the grid (step B)
        let f_we_exp_cr_grid = if E_exp_cr_grid_an == 0.0 {
            // No energy exported to grid
            RenNren::new() // ren: 0.0, nren: 0.0
        } else {
            let mut fpB_grid_i = fp_cr
                .iter()
                .filter(|fp| fp.dest == Dest::to_grid && fp.step == Step::B);
            exp_generators
                .iter()
                .fold(Ok(RenNren::new()), |acc: Result<RenNren, Error>, &gen| {
                    let acc_ok = acc?;
                    let fpB_grid_i_gen = fpB_grid_i
                        .find(|fp|
                    // NOTE: Convert from CSubtype (TCOMPONENT type) to Source (WFACTOR)
                        match *gen {
                            CSubtype::INSITU => fp.source == Source::INSITU,
                            CSubtype::COGENERACION => fp.source == Source::COGENERACION,
                            _ => false,
                        })
                        .ok_or_else(|| {
                            format_err!(
                                "No weighting factor found for: '{}, {}, to_grid, B'",
                                CURRENTCARRIER,
                                gen
                            )
                        })?;
                    Ok(acc_ok + (fpB_grid_i_gen.factors() * F_pr_i[gen]))
                })? // suma de todos los i: fpB_grid_i * F_pr_i[gen];
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
    let E_we_cr_an_A: RenNren = E_we_del_cr_an - E_we_exp_cr_an_A;

    // * Total result for step B
    // Partial result for carrier (formula 2)
    let E_we_cr_an: RenNren = E_we_del_cr_an - E_we_exp_cr_an;

    Ok(CarrierBalance {
        used_EPB: E_EPus_cr_t,
        used_nEPB: E_nEPus_cr_t,
        produced_bygen: E_pr_cr_pr_i_t,
        produced_bygen_an: E_pr_cr_pr_i_an,
        produced: E_pr_cr_t,
        produced_an: E_pr_cr_an,
        f_match: f_match_t, // load matching factor
        // E_pr_cr_used_EPus_t <- produced_used_EPus
        // E_pr_cr_i_used_EPus_t <- produced_used_EPus_bygen
        exported: E_exp_cr_t, // exp_used_nEPus + exp_grid
        exported_an: E_exp_cr_an,
        exported_bygen: E_exp_cr_pr_i_t,
        exported_bygen_an: E_exp_cr_pr_i_an,
        exported_grid: E_exp_cr_grid_t,
        exported_grid_an: E_exp_cr_grid_an,
        exported_nEPB: E_exp_cr_used_nEPus_t,
        exported_nEPB_an: E_exp_cr_used_nEPus_an,
        delivered_grid: E_del_cr_t,
        delivered_grid_an: E_del_cr_an,
        // Weighted energy: { ren, nren }
        we_delivered_grid_an: E_we_del_cr_grid_an,
        we_delivered_prod_an: E_we_del_cr_pr_an,
        we_delivered_an: E_we_del_cr_an,
        we_exported_an_A: E_we_exp_cr_an_A,
        we_exported_nEPB_an_AB: E_we_exp_cr_used_nEPus_an_AB,
        we_exported_grid_an_AB: E_we_exp_cr_grid_an_AB,
        we_exported_an_AB: E_we_exp_cr_an_AB,
        we_exported_an: E_we_exp_cr_an,
        we_an_A: E_we_cr_an_A,
        we_an: E_we_cr_an,
    })
}

// Compute overall energy performance by aggregating results for all energy carriers
//
//
pub fn energy_performance(
    components: TComponents,
    wfactors: TFactors,
    k_exp: f32,
    arearef: f32,
) -> Result<TBalance, Error> {
    // TODO:
    if arearef < 1e-3 {
        bail!("Reference area can't be zero or almost zero");
    }

    let carriers = components.cdata.clone();
    let fps = wfactors.wdata.clone();
    let mut carriers_set: Vec<Carrier> = carriers.iter().map(|e| e.carrier).collect();
    carriers_set.sort_unstable();
    carriers_set.dedup();

    // Compute balance for each carrier
    let mut balance_cr_i: HashMap<Carrier, CarrierBalance> = HashMap::new();
    for &carrier in &carriers_set {
        let cr_i: Vec<TComponent> = carriers
            .iter()
            .filter(|e| e.carrier == carrier)
            .cloned()
            .collect();
        let fp_cr: Vec<TFactor> = fps.iter()
            .filter(|e| e.carrier == carrier)
            .cloned()
            .collect();
        let bal = balance_cr(&cr_i, &fp_cr, k_exp)?;
        balance_cr_i.insert(carrier, bal);
    }

    // Accumulate partial balance values for total balance
    let balance: TTotalBalance = carriers_set.iter().fold(
        TTotalBalance::default(),
        |mut acc, cr| {
            // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step A
            acc.A = acc.A + balance_cr_i[cr].we_an_A;
            // E_we_an =  E_we_del_an - E_we_exp_an; // formula 2 step B
            acc.B = acc.B + balance_cr_i[cr].we_an;
            // Weighted energy partials
            acc.we_del = acc.we_del + balance_cr_i[cr].we_delivered_an;
            acc.we_exp_A = acc.we_exp_A + balance_cr_i[cr].we_exported_an_A;
            acc.we_exp = acc.we_exp + balance_cr_i[cr].we_exported_an;
            acc
        },
    );

    // Compute area weighted total balance
    let k_area = 1.0 / arearef;
    let balance_m2 = TTotalBalance {
        A: k_area * balance.A,
        B: k_area * balance.B,
        we_del: k_area * balance.we_del,
        we_exp_A: k_area * balance.we_exp_A,
        we_exp: k_area * balance.we_exp,
    };

    // Global data and results
    Ok(TBalance {
        components,
        wfactors,
        k_exp,
        arearef,
        balance_cr_i,
        balance,
        balance_m2,
    })
}
