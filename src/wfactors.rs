// Copyright (c) 2018-2023  Ministerio de Fomento
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
Factores de paso (weighting factors)
====================================

Define el tipo Factors (lista de Factores + Metadatos).

*/

use std::collections::HashSet;
use std::fmt;
use std::str;

use serde::{Deserialize, Serialize};

use crate::{
    error::{EpbdError, Result},
    types::{Carrier, Dest, Factor, Meta, MetaVec, RenNrenCo2, Source, Step},
    Components,
};

// --------------------------- Factors

/// Lista de factores de paso con sus metadatos
///
/// List of weighting factors bundled with its metadata
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Factors {
    /// Weighting factors list
    pub wmeta: Vec<Meta>,
    /// Metadata
    pub wdata: Vec<Factor>,
}

impl Factors {
    /// Find weighting factor
    ///
    /// * `fp_cr` - weighting factor list for a given energy carrier where search is done
    /// * `source` - match this energy source (`RED`, `INSITU`, `COGEN`)
    /// * `dest` - match this energy destination (use)
    /// * `step` - match this calculation step
    pub fn find(&self, cr: Carrier, source: Source, dest: Dest, step: Step) -> Result<RenNrenCo2> {
        self.wdata
            .iter()
            .find(|fp| {
                fp.carrier == cr && fp.source == source && fp.dest == dest && fp.step == step
            })
            .map(|fp| fp.factors())
            .ok_or_else(|| {
                EpbdError::MissingFactor(format!("'{}, {}, {}, {}'", cr, source, dest, step))
            })
    }

    /// Actualiza o establece valores de un factor de paso
    pub fn update_wfactor(
        &mut self,
        carrier: Carrier,
        source: Source,
        dest: Dest,
        step: Step,
        values: RenNrenCo2,
        comment: &str,
    ) {
        if let Some(factor) = self.wdata.iter_mut().find(|f| {
            f.carrier == carrier && f.source == source && f.step == step && f.dest == dest
        }) {
            factor.set_values(&values);
        } else {
            self.wdata
                .push(Factor::new(carrier, source, dest, step, values, comment));
        };
    }

    /// Asegura que existe un factor de paso. Si ya existe no se modifica
    pub fn ensure_wfactor(
        &mut self,
        carrier: Carrier,
        source: Source,
        dest: Dest,
        step: Step,
        values: RenNrenCo2,
        comment: &str,
    ) {
        if !self
            .wdata
            .iter()
            .any(|f| f.carrier == carrier && f.source == source && f.step == step && f.dest == dest)
        {
            self.wdata
                .push(Factor::new(carrier, source, dest, step, values, comment));
        };
    }

    /// Actualiza los factores definibles por el usuario (cogen_to_grid, cogen_to_nepb, red1 y red2)
    pub fn set_user_wfactors(mut self, user: UserWF<Option<RenNrenCo2>>) -> Self {
        use Carrier::{RED1, RED2};
        use Dest::SUMINISTRO;
        use Source::RED;
        use Step::A;

        [
            (RED1, RED, SUMINISTRO, A, user.red1, "Factor de usuario"),
            (RED2, RED, SUMINISTRO, A, user.red2, "Factor de usuario"),
        ]
        .iter()
        .for_each(|(carrier, source, dest, step, uservalue, comment)| {
            if let Some(value) = *uservalue {
                self.update_wfactor(*carrier, *source, *dest, *step, value, comment)
            }
        });

        self
    }

    /// Asegura consistencia de factores de paso definidos y deduce algunos de los que falten.
    ///
    /// Realiza los siguientes pasos:
    /// - asegura definición de factores de producción in situ
    /// - asegura definición de factores desde la red para todos los vectores
    /// - asegura que factor paso A para suministro de cogeneración es 0.0 (se considera en vector sourceal)
    /// - asegura definición de factores a la red para vectores con exportación
    /// - asegura que existe RED1 | RED2 en suministro
    ///
    /// TODO: refactorizar moviendo algunos métodos a trait CteFactorsExt
    pub fn normalize(mut self, defaults: &UserWF<RenNrenCo2>) -> Result<Self> {
        use Carrier::*;
        use Dest::*;
        use Source::*;
        use Step::*;

        // Vectores existentes
        let wf_carriers: HashSet<_> = self.wdata.iter().map(|f| f.carrier).collect();

        // Asegura que existe EAMBIENTE, INSITU, SUMINISTRO, A, 1.0, 0.0
        self.update_wfactor(
            EAMBIENTE,
            INSITU,
            SUMINISTRO,
            A,
            RenNrenCo2::new(1.0, 0.0, 0.0),
            "Recursos usados para obtener energía ambiente",
        );

        // Asegura que existe EAMBIENTE, RED, SUMINISTRO, A, 1.0, 0.0
        self.update_wfactor(
            EAMBIENTE,
            RED,
            SUMINISTRO,
            A,
            RenNrenCo2::new(1.0, 0.0, 0.0),
            "Recursos usados para obtener energía ambiente (red ficticia)",
        );

        // Asegura que existe TERMOSOLAR, INSITU, SUMINISTRO, A, 1.0, 0.0
        self.update_wfactor(
            TERMOSOLAR,
            INSITU,
            SUMINISTRO,
            A,
            RenNrenCo2::new(1.0, 0.0, 0.0),
            "Recursos usados para obtener energía solar térmica",
        );

        // Asegura que existe TERMOSOLAR, RED, SUMINISTRO, A, 1.0, 0.0
        self.update_wfactor(
            TERMOSOLAR,
            RED,
            SUMINISTRO,
            A,
            RenNrenCo2::new(1.0, 0.0, 0.0),
            "Recursos usados para obtener energía solar térmica (red ficticia)",
        );

        // Asegura que existe ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.0, 0.0 si hay ELECTRICIDAD
        if wf_carriers.contains(&ELECTRICIDAD) {
            self.update_wfactor(
                ELECTRICIDAD,
                INSITU,
                SUMINISTRO,
                A,
                RenNrenCo2::new(1.0, 0.0, 0.0),
                "Recursos usados para generar electricidad in situ",
            );
        }

        // Asegura definición de factores de red para todos los vectores energéticos
        let has_grid_factors_for_all_carriers = wf_carriers.iter().all(|&c| {
            self.wdata.iter().any(|f| {
                f.carrier == c
                    && f.source == Source::RED
                    && f.dest == Dest::SUMINISTRO
                    && f.step == Step::A
            })
        });
        if !has_grid_factors_for_all_carriers {
            return Err(EpbdError::MissingFactor(
                "Factores de red VECTOR, INSITU, SUMINISTRO, A, fren?, fnren?".into(),
            ));
        }

        // // En paso A, el factor SUMINISTRO de cogeneración es 0.0, 0.0 ya que el impacto se tiene en cuenta en el suministro del vector de generación
        // self.update_wfactor(
        //     ELECTRICIDAD,
        //     COGEN,
        //     SUMINISTRO,
        //     A,
        //     RenNrenCo2::new(0.0, 0.0, 0.0),
        //     "Factor de paso generado (el impacto de la cogeneración se tiene en cuenta en el vector de suministro)",
        // );

        // Asegura que todos los vectores con exportación tienen factores de paso a la red y a usos no EPB
        let exp_carriers = [
            (Carrier::ELECTRICIDAD, Source::INSITU),
            // (Carrier::ELECTRICIDAD, Source::COGEN),
            (Carrier::EAMBIENTE, Source::INSITU),
            (Carrier::TERMOSOLAR, Source::INSITU),
        ];
        for (c, s) in &exp_carriers {
            // Asegura que existe VECTOR, SRC, A_RED | A_NEPB, A, ren, nren
            let fp_a_input = self
                .wdata
                .iter()
                .find(|f| {
                    f.carrier == *c
                        && f.source == *s
                        && f.step == Step::A
                        && f.dest == Dest::SUMINISTRO
                })
                .map(|f| f.factors());

            if let Some(factors) = fp_a_input {
                // VECTOR, SRC, A_RED, A, ren, nren === VECTOR, SRC, SUMINISTRO, A, ren, nren
                self.ensure_wfactor(
                    *c,
                    *s,
                    A_RED,
                    A,
                    factors,
                    "Recursos usados para producir la energía exportada a la red",
                );
                // VECTOR, SRC, A_NEPB, A, ren, nren == VECTOR, SRC, SUMINISTRO, A, ren, nren
                self.ensure_wfactor(
                    *c,
                    *s,
                    A_NEPB,
                    A,
                    factors,
                    "Recursos usados para producir la energía exportada a usos no EPB",
                );
            }

            // Asegura que existe VECTOR, SRC, A_RED | A_NEPB, B, ren, nren
            let fp_a_red_input = self
                .wdata
                .iter()
                .find(|f| {
                    f.carrier == *c
                        && f.source == Source::RED
                        && f.dest == Dest::SUMINISTRO
                        && f.step == Step::A
                })
                .map(|f| f.factors());

            if let Some(factors) = fp_a_red_input {
                // VECTOR, SRC, A_RED, B, ren, nren == VECTOR, RED, SUMINISTRO, A, ren, nren
                self.ensure_wfactor(
                    *c,
                    *s,
                    A_RED,
                    B,
                    factors,
                    "Recursos ahorrados a la red por la energía producida in situ y exportada a la red",
                );
                // VECTOR, SRC, A_NEPB, B, ren, nren == VECTOR, RED, SUMINISTRO, A, ren, nren
                self.ensure_wfactor(
                    *c,
                    *s,
                    A_NEPB,
                    B,
                    factors,
                    "Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB",
                );
            } else {
                return Err(EpbdError::MissingFactor(format!("{}, SUMINISTRO, A", c)));
            }
        }

        // Asegura que existe RED1 | RED2, RED, SUMINISTRO, A, ren, nren
        self.ensure_wfactor(
            RED1,
            RED,
            SUMINISTRO,
            A,
            defaults.red1,
            "Recursos usados para suministrar energía de la red de distrito 1 (definible por el usuario)",
        );

        self.ensure_wfactor(
            RED2,
            RED,
            SUMINISTRO,
            A,
            defaults.red2,
            "Recursos usados para suministrar energía de la red de distrito 2 (definible por el usuario)",
        );

        Ok(self)
    }

    /// Elimina factores de paso no usados en los datos de vectores energéticos.
    ///
    /// Elimina los factores:
    ///  - de vectores que no aparecen en los datos
    ///  - de cogeneración si no hay cogeneración
    ///  - para exportación a usos no EPB si no se aparecen en los datos
    ///  - de electricidad in situ si no aparece una producción de ese tipo
    pub fn strip(mut self, components: &Components) -> Self {
        let wf_carriers = components.available_carriers();
        // Mantenemos factores para todos los vectores usados
        self.wdata.retain(|f| wf_carriers.contains(&f.carrier));
        // Mantenemos factores para cogeneración sólo si hay cogeneración
        let has_cogen = components.data.iter().any(|c| c.is_cogen_pr());
        self.wdata
            .retain(|f| f.source != Source::COGEN || has_cogen);
        // Mantenemos factores a usos no EPB si hay uso de no EPB
        let has_nepb = components.data.iter().any(|c| c.is_nepb_use());
        self.wdata.retain(|f| f.dest != Dest::A_NEPB || has_nepb);
        // Mantenemos factores de electricidad in situ si no hay producción de ese tipo
        let has_elec_onsite = components
            .data
            .iter()
            .any(|c| c.is_electricity() && c.is_onsite_pr());
        self.wdata.retain(|f| {
            f.carrier != Carrier::ELECTRICIDAD || f.source != Source::INSITU || has_elec_onsite
        });
        self
    }

    /// Convierte factores de paso con perímetro "distant" a factores de paso "nearby".
    ///
    /// Los elementos que tiene origen en la RED (!= INSITU, != COGEN)
    /// y no están en la lista nearby_list cambian sus factores de paso
    /// de forma que ren' = 0 y nren' = ren + nren.
    /// **ATENCIÓN**: ¡¡La producción eléctrica de la cogeneración entra con (factores ren:0, nren:0)!!
    pub fn to_nearby(&self, nearby_list: &[Carrier]) -> Self {
        let wmeta = self.wmeta.clone();
        let mut wdata: Vec<Factor> = Vec::new();

        for f in self.wdata.iter().cloned() {
            if f.source == Source::INSITU
                || f.source == Source::COGEN
                || nearby_list.contains(&f.carrier)
            {
                wdata.push(f)
            } else {
                wdata.push(Factor::new(
                    f.carrier,
                    f.source,
                    f.dest,
                    f.step,
                    RenNrenCo2::new(0.0, f.ren + f.nren, f.co2), // ¿Esto es lo que tiene más sentido?
                    format!("Perímetro nearby: {}", f.comment),
                ))
            }
        }
        let mut factors = Factors { wmeta, wdata };
        factors.set_meta("CTE_PERIMETRO", "NEARBY");
        factors
    }

    /// Incorpora factores de exportación de la electricidad cogenerada
    ///
    /// Devuelve a definición de los factores de exportación a NEPB y RED (paso A y paso B),
    /// para la electricidad cogenerada, que pueden ser agregados directamente a Factors.wdata
    ///
    /// También devuelve las estructuras de datos de los factores de exportación paso A
    /// para el perímetro distante y próximo, para facilitar el cálculo de RER_nrb
    #[allow(non_snake_case)]
    pub(crate) fn add_cgn_factors(&mut self, components: &Components) -> Result<()> {
        let fP_exp_el_cgn_A = match self.compute_cgn_exp_fP_A(components, false)? {
            Some(fP) => fP,
            _ => return Ok(()),
        };

        // Factores derivados para el paso A (recursos usados)
        let factor_input_A = Factor::new(
            Carrier::ELECTRICIDAD,
            Source::COGEN,
            Dest::SUMINISTRO,
            Step::A,
            fP_exp_el_cgn_A,
            "Recursos usados para el suministrar electricidad cogenerada (calculado)",
        );

        // Factores derivados para el paso A (recursos usados)
        let factor_to_nepb_A = Factor::new(
            Carrier::ELECTRICIDAD,
            Source::COGEN,
            Dest::A_NEPB,
            Step::A,
            fP_exp_el_cgn_A,
            "Recursos usados para la exportación a usos no EPB (calculado)",
        );
        let factor_to_grid_A = Factor::new(
            Carrier::ELECTRICIDAD,
            Source::COGEN,
            Dest::A_RED,
            Step::A,
            fP_exp_el_cgn_A,
            "Recursos usados para la exportación a la red (calculado)",
        );

        // Factores derivados para el paso B (recursos ahorrados a la red, iguales al paso A de red)
        let fP_el_grid_A = self.find(
            Carrier::ELECTRICIDAD,
            Source::RED,
            Dest::SUMINISTRO,
            Step::A,
        )?;
        let factor_to_nepb_B = Factor::new(
            Carrier::ELECTRICIDAD,
            Source::COGEN,
            Dest::A_NEPB,
            Step::B,
            fP_el_grid_A,
            "Recursos ahorrados a la red por la exportación a usos no EPB (calculado)",
        );
        let factor_to_grid_B = Factor::new(
            Carrier::ELECTRICIDAD,
            Source::COGEN,
            Dest::A_RED,
            Step::B,
            fP_el_grid_A,
            "Recursos ahorrados a la red por la exportación a la red (calculado)",
        );

        // Incorporamos los factores a wfactors
        self.wdata.push(factor_input_A);
        self.wdata.push(factor_to_nepb_A);
        self.wdata.push(factor_to_grid_A);
        self.wdata.push(factor_to_nepb_B);
        self.wdata.push(factor_to_grid_B);

        Ok(())
    }

    #[allow(non_snake_case)]
    pub(crate) fn compute_cgn_exp_fP_A(
        &self,
        components: &Components,
        only_nearby: bool,
    ) -> Result<Option<RenNrenCo2>> {
        // Si hay producción eléctrica
        // Calcula f_exp_pr_el_A_chp_t = suma (E_in_t * f_in_t) / pr_el_chp_t
        use crate::types::Energy;
        use crate::vecops::vecvecsum;
        use std::collections::HashMap;
        let mut prod = Vec::<f32>::new();
        let mut used = HashMap::<Carrier, Vec<f32>>::new();
        for c in &components.data {
            match c {
                Energy::Used(e) if c.is_cogen_use() => {
                    used.entry(e.carrier)
                        .and_modify(|item| *item = vecvecsum(item, &e.values))
                        .or_insert_with(|| e.values.clone());
                }
                Energy::Prod(e) if c.is_cogen_pr() => {
                    prod = if prod.is_empty() {
                        e.values.clone()
                    } else {
                        vecvecsum(&prod, &e.values)
                    }
                }
                _ => continue,
            }
        }
        if prod.is_empty() {
            return Ok(None);
        }
        if used.is_empty() {
            return Err(EpbdError::WrongInput(
                "No se han definido los consumos para la cogeneración".into(),
            ));
        };
        let mut fP_exp_el_cgn_A = RenNrenCo2::default();
        for (carrier, used_t) in used {
            if only_nearby && !carrier.is_nearby() {
                continue;
            }
            let fP_A_cr = self.find(carrier, Source::RED, Dest::SUMINISTRO, Step::A)?;
            let used_prod_ratio_sum = used_t
                .iter()
                .zip(prod.iter())
                .map(|(us, pr)| if *pr > 0.0 { us / pr } else { 0.0 })
                .sum::<f32>();
            fP_exp_el_cgn_A += fP_A_cr * used_prod_ratio_sum;
        }
        Ok(Some(fP_exp_el_cgn_A))
    }
}

impl MetaVec for Factors {
    fn get_metavec(&self) -> &Vec<Meta> {
        &self.wmeta
    }
    fn get_mut_metavec(&mut self) -> &mut Vec<Meta> {
        &mut self.wmeta
    }
}

impl fmt::Display for Factors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metalines = self
            .wmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self
            .wdata
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n{}", metalines, datalines)
    }
}

impl str::FromStr for Factors {
    type Err = EpbdError;

    fn from_str(s: &str) -> std::result::Result<Factors, Self::Err> {
        let lines: Vec<&str> = s.lines().map(str::trim).collect();
        let metalines = lines
            .iter()
            .filter(|l| l.starts_with("#META") || l.starts_with("#CTE_"));
        let datalines = lines
            .iter()
            .filter(|l| !(l.starts_with('#') || l.starts_with("vector,") || l.is_empty()));
        let wmeta = metalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Meta>>>()?;
        let wdata = datalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Factor>>>()?;
        Ok(Factors { wmeta, wdata })
    }
}

/// Estructura para definir valores por defecto y valores de usuario
#[derive(Debug, Copy, Clone)]
pub struct UserWF<T = RenNrenCo2> {
    /// Factores de paso de redes de distrito 1.
    /// RED1, RED, SUMINISTRO, A, ren, nren
    pub red1: T,
    /// Factores de paso de redes de distrito 2.
    /// RED2, RED, SUMINISTRO, A, ren, nren
    pub red2: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tfactors() {
        let tfactors1 = "#META CTE_FUENTE: RITE2014
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954, 0.331 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para producir electricidad in situ";

        // roundtrip building from/to string
        assert_eq!(tfactors1.parse::<Factors>().unwrap().to_string(), tfactors1);
    }

    #[test]
    fn set_user_factors() {
        let tfactors1 = "#META CTE_FUENTE: RITE2014
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954, 0.331 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para producir electricidad in situ
".parse::<Factors>().unwrap();
        let tfactorsres = "#META CTE_FUENTE: RITE2014
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954, 0.331 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para producir electricidad in situ
RED1, RED, SUMINISTRO, A, 0.100, 0.125, 0.500 # Factor de usuario
RED2, RED, SUMINISTRO, A, 0.125, 0.100, 0.500 # Factor de usuario";
        assert_eq!(
            tfactors1
                .set_user_wfactors(UserWF {
                    red1: Some(RenNrenCo2::new(0.1, 0.125, 0.5)),
                    red2: Some(RenNrenCo2::new(0.125, 0.1, 0.5)),
                })
                .to_string(),
            tfactorsres
        );
    }

    #[test]
    fn normalize_and_strip() {
        let tfactors = "#META CTE_FUENTE: RITE2014
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954, 0.331 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para producir electricidad in situ
".parse::<Factors>().unwrap();
        let tfactors_normalized_str = "#META CTE_FUENTE: RITE2014
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954, 0.331 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para producir electricidad in situ
EAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para obtener energía ambiente
EAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para obtener energía ambiente (red ficticia)
TERMOSOLAR, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para obtener energía solar térmica
TERMOSOLAR, RED, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para obtener energía solar térmica (red ficticia)
ELECTRICIDAD, INSITU, A_RED, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a la red
ELECTRICIDAD, INSITU, A_NEPB, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a usos no EPB
ELECTRICIDAD, INSITU, A_RED, B, 0.414, 1.954, 0.331 # Recursos ahorrados a la red por la energía producida in situ y exportada a la red
ELECTRICIDAD, INSITU, A_NEPB, B, 0.414, 1.954, 0.331 # Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB
EAMBIENTE, INSITU, A_RED, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a la red
EAMBIENTE, INSITU, A_NEPB, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a usos no EPB
EAMBIENTE, INSITU, A_RED, B, 1.000, 0.000, 0.000 # Recursos ahorrados a la red por la energía producida in situ y exportada a la red
EAMBIENTE, INSITU, A_NEPB, B, 1.000, 0.000, 0.000 # Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB
TERMOSOLAR, INSITU, A_RED, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a la red
TERMOSOLAR, INSITU, A_NEPB, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a usos no EPB
TERMOSOLAR, INSITU, A_RED, B, 1.000, 0.000, 0.000 # Recursos ahorrados a la red por la energía producida in situ y exportada a la red
TERMOSOLAR, INSITU, A_NEPB, B, 1.000, 0.000, 0.000 # Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB
RED1, RED, SUMINISTRO, A, 0.000, 1.300, 0.300 # Recursos usados para suministrar energía de la red de distrito 1 (definible por el usuario)
RED2, RED, SUMINISTRO, A, 0.000, 1.300, 0.300 # Recursos usados para suministrar energía de la red de distrito 2 (definible por el usuario)";
        let tcomps = "CONSUMO, ILU, ELECTRICIDAD, 1 # Solo consume electricidad de red"
            .parse::<Components>()
            .unwrap();
        let tfactors_normalized_stripped_str = "#META CTE_FUENTE: RITE2014
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954, 0.331 # Recursos usados para suministrar electricidad (peninsular) desde la red";

        let tfactors_normalized = tfactors
            .normalize(&UserWF {
                red1: RenNrenCo2 {
                    ren: 0.0,
                    nren: 1.3,
                    co2: 0.3,
                },
                red2: RenNrenCo2 {
                    ren: 0.0,
                    nren: 1.3,
                    co2: 0.3,
                },
            })
            .unwrap();
        let tfactors_normalized_stripped = tfactors_normalized.clone().strip(&tcomps);

        assert_eq!(tfactors_normalized.to_string(), tfactors_normalized_str);
        assert_eq!(
            tfactors_normalized_stripped.to_string(),
            tfactors_normalized_stripped_str
        );
    }
}
