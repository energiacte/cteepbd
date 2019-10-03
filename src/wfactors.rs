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
Weighting factors
=================

Define Factors typr (Factor list + Metadata).

*/

use std::collections::HashSet;
use std::fmt;
use std::str;

use crate::{
    error::EpbdError,
    types::{CSubtype, Carrier, Dest, Factor, Meta, MetaVec, RenNrenCo2, Source, Step},
    Components,
};

// --------------------------- Factors

/// List of weighting factors bundled with its metadata
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Factors {
    /// Weighting factors list
    pub wmeta: Vec<Meta>,
    /// Metadata
    pub wdata: Vec<Factor>,
}

impl Factors {
    /// Remove non EPB weighting factors from the factor list
    pub fn strip_nepb(&mut self) {
        self.wdata.retain(|e| e.dest != Dest::A_NEPB);
    }

    /// Actualiza los factores definibles por el usuario (cogen_to_grid, cogen_to_nepb, red1 y red2)
    pub fn set_user_wfactors(mut self, user: &UserWF<Option<RenNrenCo2>>) -> Self {
        // ------ Cogeneración a red ----------
        if let Some(ucog) = user.cogen_to_grid {
            if let Some(factor) = self.wdata.iter_mut().find(|f| {
                f.source == Source::COGENERACION && f.step == Step::A && f.dest == Dest::A_RED
            }) {
                factor.set_values(&ucog);
            } else {
                self.wdata.push(Factor::new(
                    Carrier::ELECTRICIDAD,
                    Source::COGENERACION,
                    Dest::A_RED,
                    Step::A,
                    ucog.ren,
                    ucog.nren,
                    ucog.co2,
                    "Factor de usuario",
                ));
            };
        };

        // ------ Cogeneración a usos no EPB ----------
        if let Some(ucog) = user.cogen_to_nepb {
            if let Some(factor) = self.wdata.iter_mut().find(|f| {
                f.source == Source::COGENERACION && f.step == Step::A && f.dest == Dest::A_NEPB
            }) {
                factor.set_values(&ucog);
            } else {
                self.wdata.push(Factor::new(
                    Carrier::ELECTRICIDAD,
                    Source::COGENERACION,
                    Dest::A_NEPB,
                    Step::A,
                    ucog.ren,
                    ucog.nren,
                    ucog.co2,
                    "Factor de usuario",
                ));
            };
        };

        // ------ Red1 ----------
        if let Some(ured1) = user.red1 {
            if let Some(factor) = self.wdata.iter_mut().find(|f| {
                f.carrier == Carrier::RED1 && f.step == Step::A && f.dest == Dest::SUMINISTRO
            }) {
                factor.set_values(&ured1);
            } else {
                self.wdata.push(Factor::new(
                    Carrier::RED1,
                    Source::RED,
                    Dest::SUMINISTRO,
                    Step::A,
                    ured1.ren,
                    ured1.nren,
                    ured1.co2,
                    "Factor de usuario",
                ));
            };
        };

        // ------ Red2 ----------
        if let Some(ured2) = user.red2 {
            if let Some(factor) = self.wdata.iter_mut().find(|f| {
                f.carrier == Carrier::RED2 && f.step == Step::A && f.dest == Dest::SUMINISTRO
            }) {
                factor.set_values(&ured2);
            } else {
                self.wdata.push(Factor::new(
                    Carrier::RED2,
                    Source::RED,
                    Dest::SUMINISTRO,
                    Step::A,
                    ured2.ren,
                    ured2.nren,
                    ured2.co2,
                    "Factor de usuario",
                ));
            };
        };
        self
    }

    /// Asegura consistencia de factores de paso definidos y deduce algunos de los que falten.
    ///
    /// Realiza los siguientes pasos:
    /// - asegura definición de factores de producción in situ
    /// - asegura definición de factores desde la red para todos los vectores
    /// - asegura que factor paso A para suministro de cogeneración es 0.0 (se considera en vector original)
    /// - asegura definición de factores a la red para vectores con exportación
    /// - asegura que existe RED1 | RED2 en suministro
    ///
    /// TODO: se deberían separar algunos de estos pasos como métodos de CteFactorsExt
    pub fn normalize(mut self, defaults: &UserWF<RenNrenCo2>) -> Result<Self, EpbdError> {
        // Vectores existentes
        let wf_carriers: HashSet<_> = self.wdata.iter().map(|f| f.carrier).collect();

        // Asegura que existe MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.0, 0.0
        let has_ma_insitu_input_a = self.wdata.iter().any(|f| {
            f.carrier == Carrier::MEDIOAMBIENTE
                && f.source == Source::INSITU
                && f.dest == Dest::SUMINISTRO
                && f.step == Step::A
        });
        if !has_ma_insitu_input_a {
            self.wdata.push(Factor::new(
                Carrier::MEDIOAMBIENTE,
                Source::INSITU,
                Dest::SUMINISTRO,
                Step::A,
                1.0,
                0.0,
                0.0,
                "Recursos usados para obtener energía térmica del medioambiente",
            ));
        }
        // Asegura que existe MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.0, 0.0
        let has_ma_red_input_a = self.wdata.iter().any(|f| {
            f.carrier == Carrier::MEDIOAMBIENTE
                && f.source == Source::RED
                && f.dest == Dest::SUMINISTRO
                && f.step == Step::A
        });
        if !has_ma_red_input_a {
            // MEDIOAMBIENTE, RED, SUMINISTRO, A, ren, nren === MEDIOAMBIENTE, INSITU, SUMINISTRO, A, ren, nren
            self.wdata.push(Factor::new(
                Carrier::MEDIOAMBIENTE,
                Source::RED,
                Dest::SUMINISTRO,
                Step::A,
                1.0,
                0.0,
                0.0,
                "Recursos usados para obtener energía térmica del medioambiente (red ficticia)",
            ));
        }
        // Asegura que existe ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.0, 0.0 si hay ELECTRICIDAD
        let has_elec_and_elec_insitu_input_a = wf_carriers.contains(&Carrier::ELECTRICIDAD)
            && !self.wdata.iter().any(|f| {
                f.carrier == Carrier::ELECTRICIDAD
                    && f.source == Source::INSITU
                    && f.dest == Dest::SUMINISTRO
            });
        if has_elec_and_elec_insitu_input_a {
            self.wdata.push(Factor::new(
                Carrier::ELECTRICIDAD,
                Source::INSITU,
                Dest::SUMINISTRO,
                Step::A,
                1.0,
                0.0,
                0.0,
                "Recursos usados para generar electricidad in situ",
            ));
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
                "factores de red VECTOR, INSITU, SUMINISTRO, A, fren?, fnren?".into(),
            ));
        }
        // En paso A, el factor SUMINISTRO de cogeneración es 0.0, 0.0 ya que el impacto se tiene en cuenta en el suministro del vector de generación
        let has_cogen_input = self
            .wdata
            .iter()
            .any(|f| f.source == Source::COGENERACION && f.dest == Dest::SUMINISTRO);
        if !has_cogen_input {
            self.wdata.push(Factor::new(
                Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::SUMINISTRO, Step::A, 0.0, 0.0, 0.0,
                "Factor de paso generado (el impacto de la cogeneración se tiene en cuenta en el vector de suministro)"));
        }
        // Asegura que todos los vectores con exportación tienen factores de paso a la red y a usos no EPB
        let exp_carriers = [
            (Carrier::ELECTRICIDAD, Source::INSITU),
            (Carrier::ELECTRICIDAD, Source::COGENERACION),
            (Carrier::MEDIOAMBIENTE, Source::INSITU),
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
                .and_then(|f| Some(f.clone()));

            let has_to_grid = self.wdata.iter().any(|f| {
                f.carrier == *c && f.source == *s && f.step == Step::A && f.dest == Dest::A_RED
            });
            if !has_to_grid {
                if *s != Source::COGENERACION {
                    // VECTOR, SRC, A_RED, A, ren, nren === VECTOR, SRC, SUMINISTRO, A, ren, nren
                    if fp_a_input.is_some() {
                        let f = fp_a_input.as_ref().unwrap();
                        self.wdata.push(Factor {
                            dest: Dest::A_RED,
                            step: Step::A,
                            comment: "Recursos usados para producir la energía exportada a la red"
                                .to_string(),
                            ..*f
                        });
                    } else {
                        return Err(EpbdError::MissingFactor(format!(
                            "suministro del vector {} para definir exportación a la red en paso A",
                            c
                        )));
                    }
                } else {
                    // TODO: Igual aquí hay que indicar que se deben definir factores de usuario en un bail y no hacer nada
                    // Asegura que existe ELECTRICIDAD, COGENERACION, A_RED, A, ren, nren - ver 9.6.6.2.3
                    let has_cogen_to_grid = self.wdata.iter().any(|f| {
                        f.carrier == Carrier::ELECTRICIDAD
                            && f.source == Source::COGENERACION
                            && f.dest == Dest::A_RED
                            && f.step == Step::A
                    });
                    if !has_cogen_to_grid {
                        let cogen = defaults.cogen_to_grid;
                        self.wdata.push(Factor::new(
                        Carrier::ELECTRICIDAD, Source::COGENERACION, Dest::A_RED, Step::A, cogen.ren, cogen.nren, cogen.co2,
                        "Recursos usados para producir electricidad cogenerada y vertida a la red. Valor predefinido"));
                    }
                }
            }
            let has_to_nepb = self.wdata.iter().any(|f| {
                f.carrier == *c && f.source == *s && f.step == Step::A && f.dest == Dest::A_NEPB
            });
            if !has_to_nepb {
                if *s != Source::COGENERACION {
                    // VECTOR, SRC, A_NEPB, A, ren, nren == VECTOR, SRC, SUMINISTRO, A, ren, nren
                    if fp_a_input.is_some() {
                        let f = fp_a_input.as_ref().unwrap();
                        self.wdata.push(Factor {
                            dest: Dest::A_NEPB,
                            step: Step::A,
                            comment:
                                "Recursos usados para producir la energía exportada a usos no EPB"
                                    .to_string(),
                            ..*f
                        });
                    } else {
                        return Err(EpbdError::MissingFactor(format!(
                            "suministro del vector {} para definir exportación a usos no EPB en paso A",
                            c
                        )));
                    }
                } else {
                    // TODO: Igual aquí hay que indicar que se deben definir factores de usuario en un bail y no hacer nada
                    // TODO: Si está definido para A_RED (no por defecto) y no para A_NEPB, qué hacemos? usamos por defecto? usamos igual a A_RED?
                    // Asegura que existe ELECTRICIDAD, COGENERACION, A_NEPB, A, ren, nren - ver 9.6.6.2.3
                    let has_cogen_to_nepb = self.wdata.iter().any(|f| {
                        f.carrier == Carrier::ELECTRICIDAD
                            && f.source == Source::COGENERACION
                            && f.dest == Dest::A_NEPB
                            && f.step == Step::A
                    });
                    if !has_cogen_to_nepb {
                        let cogen = defaults.cogen_to_nepb;
                        self.wdata.push(Factor::new(
                            Carrier::ELECTRICIDAD,
                            Source::COGENERACION,
                            Dest::A_NEPB,
                            Step::A,
                            cogen.ren,
                            cogen.nren,
                            cogen.co2,
                            "Valor predefinido",
                        ));
                    }
                }
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
                .and_then(|f| Some(f.clone()));
            let has_to_grid_b = self.wdata.iter().any(|f| {
                f.carrier == *c && f.source == *s && f.step == Step::B && f.dest == Dest::A_RED
            });
            if !has_to_grid_b {
                // VECTOR, SRC, A_RED, B, ren, nren == VECTOR, RED, SUMINISTRO, A, ren, nren
                if fp_a_red_input.is_some() {
                    let f = fp_a_red_input.as_ref().unwrap();
                    self.wdata.push(Factor::new(f.carrier, *s, Dest::A_RED, Step::B, f.ren, f.nren, f.co2,
                    "Recursos ahorrados a la red por la energía producida in situ y exportada a la red"));
                } else {
                    return Err(EpbdError::MissingFactor(format!(
                        "suministro del vector {} para exportación a la red en paso B",
                        c
                    )));
                }
            }
            let has_to_nepb_b = self.wdata.iter().any(|f| {
                f.carrier == *c && f.source == *s && f.step == Step::B && f.dest == Dest::A_NEPB
            });
            if !has_to_nepb_b {
                // VECTOR, SRC, A_NEPB, B, ren, nren == VECTOR, RED, SUMINISTRO, A, ren, nren
                if fp_a_red_input.is_some() {
                    let f = fp_a_red_input.as_ref().unwrap();
                    self.wdata.push(Factor::new(f.carrier, *s, Dest::A_NEPB, Step::B, f.ren, f.nren, f.co2,
                    "Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB"));
                } else {
                    return Err(EpbdError::MissingFactor(format!(
                        "suministro del vector {} para exportación a usos no EPB en paso B",
                        c
                    )));
                }
            }
        }

        // Asegura que existe RED1 | RED2, RED, SUMINISTRO, A, ren, nren
        let has_red1_red_input = self.wdata.iter().any(|f| {
            f.carrier == Carrier::RED1 && f.source == Source::RED && f.dest == Dest::SUMINISTRO
        });
        if !has_red1_red_input {
            let red1 = defaults.red1;
            self.wdata.push(Factor::new(Carrier::RED1, Source::RED, Dest::SUMINISTRO, Step::A,
            red1.ren, red1.nren, red1.co2, "Recursos usados para suministrar energía de la red de distrito 1 (definible por el usuario)"));
        }
        let has_red2_red_input = self.wdata.iter().any(|f| {
            f.carrier == Carrier::RED2 && f.source == Source::RED && f.dest == Dest::SUMINISTRO
        });
        if !has_red2_red_input {
            let red2 = defaults.red2;
            self.wdata.push(Factor::new(Carrier::RED2, Source::RED, Dest::SUMINISTRO, Step::A,
            red2.ren, red2.nren, red2.co2, "Recursos usados para suministrar energía de la red de distrito 2 (definible por el usuario)"));
        }

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
        let wf_carriers: HashSet<_> = components.cdata.iter().map(|c| c.carrier).collect();
        let has_cogen = components
            .cdata
            .iter()
            .any(|c| c.csubtype == CSubtype::COGENERACION);
        let has_nepb = components
            .cdata
            .iter()
            .any(|c| c.csubtype == CSubtype::NEPB);
        let has_elec_insitu = components
            .cdata
            .iter()
            .any(|c| c.carrier == Carrier::ELECTRICIDAD && c.csubtype == CSubtype::INSITU);
        self.wdata.retain(|f| wf_carriers.contains(&f.carrier));
        self.wdata
            .retain(|f| f.source != Source::COGENERACION || has_cogen);
        self.wdata.retain(|f| f.dest != Dest::A_NEPB || has_nepb);
        self.wdata.retain(|f| {
            f.carrier != Carrier::ELECTRICIDAD || f.source != Source::INSITU || has_elec_insitu
        });
        self
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

    fn from_str(s: &str) -> Result<Factors, Self::Err> {
        let lines: Vec<&str> = s.lines().map(str::trim).collect();
        let metalines = lines
            .iter()
            .filter(|l| l.starts_with("#META") || l.starts_with("#CTE_"));
        let datalines = lines
            .iter()
            .filter(|l| !(l.starts_with('#') || l.starts_with("vector,") || l.is_empty()));
        let wmeta = metalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Meta>, _>>()?;
        let wdata = datalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Factor>, _>>()?;
        Ok(Factors { wmeta, wdata })
    }
}

/// Estructura para definir valores por defecto y valores de usuario
#[derive(Debug)]
pub struct UserWF<T = RenNrenCo2> {
    /// Factores de paso de redes de distrito 1.
    /// RED1, RED, SUMINISTRO, A, ren, nren
    pub red1: T,
    /// Factores de paso de redes de distrito 2.
    /// RED2, RED, SUMINISTRO, A, ren, nren
    pub red2: T,
    /// Factores de paso para exportación a la red (paso A) de electricidad cogenerada.
    /// ELECTRICIDAD, COGENERACION, A_RED, A, ren, nren
    pub cogen_to_grid: T,
    /// Factores de paso para exportación a usos no EPB (paso A) de electricidad cogenerada.
    /// ELECTRICIDAD, COGENERACION, A_NEPB, A, ren, nren
    pub cogen_to_nepb: T,
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
ELECTRICIDAD, COGENERACION, A_RED, A, 0.125, 0.500, 1.000 # Factor de usuario
ELECTRICIDAD, COGENERACION, A_NEPB, A, 0.500, 0.125, 2.000 # Factor de usuario
RED1, RED, SUMINISTRO, A, 0.100, 0.125, 0.500 # Factor de usuario
RED2, RED, SUMINISTRO, A, 0.125, 0.100, 0.500 # Factor de usuario";
        assert_eq!(
            tfactors1
                .set_user_wfactors(&UserWF {
                    red1: Some(RenNrenCo2::new(0.1, 0.125, 0.5)),
                    red2: Some(RenNrenCo2::new(0.125, 0.1, 0.5)),
                    cogen_to_grid: Some(RenNrenCo2::new(0.125, 0.5, 1.0)),
                    cogen_to_nepb: Some(RenNrenCo2::new(0.5, 0.125, 2.0)),
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
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para obtener energía térmica del medioambiente
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000, 0.000 # Recursos usados para obtener energía térmica del medioambiente (red ficticia)
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000, 0.000 # Factor de paso generado (el impacto de la cogeneración se tiene en cuenta en el vector de suministro)
ELECTRICIDAD, INSITU, A_RED, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a la red\nELECTRICIDAD, INSITU, A_NEPB, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a usos no EPB
ELECTRICIDAD, INSITU, A_RED, B, 0.414, 1.954, 0.331 # Recursos ahorrados a la red por la energía producida in situ y exportada a la red
ELECTRICIDAD, INSITU, A_NEPB, B, 0.414, 1.954, 0.331 # Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB
ELECTRICIDAD, COGENERACION, A_RED, A, 0.000, 2.500, 0.300 # Recursos usados para producir electricidad cogenerada y vertida a la red. Valor predefinido
ELECTRICIDAD, COGENERACION, A_NEPB, A, 0.000, 2.500, 0.300 # Valor predefinido
ELECTRICIDAD, COGENERACION, A_RED, B, 0.414, 1.954, 0.331 # Recursos ahorrados a la red por la energía producida in situ y exportada a la red
ELECTRICIDAD, COGENERACION, A_NEPB, B, 0.414, 1.954, 0.331 # Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB
MEDIOAMBIENTE, INSITU, A_RED, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a la red
MEDIOAMBIENTE, INSITU, A_NEPB, A, 1.000, 0.000, 0.000 # Recursos usados para producir la energía exportada a usos no EPB
MEDIOAMBIENTE, INSITU, A_RED, B, 1.000, 0.000, 0.000 # Recursos ahorrados a la red por la energía producida in situ y exportada a la red
MEDIOAMBIENTE, INSITU, A_NEPB, B, 1.000, 0.000, 0.000 # Recursos ahorrados a la red por la energía producida in situ y exportada a usos no EPB
RED1, RED, SUMINISTRO, A, 0.000, 1.300, 0.300 # Recursos usados para suministrar energía de la red de distrito 1 (definible por el usuario)
RED2, RED, SUMINISTRO, A, 0.000, 1.300, 0.300 # Recursos usados para suministrar energía de la red de distrito 2 (definible por el usuario)";
        let tcomps = "ELECTRICIDAD, CONSUMO, EPB, NDEF, 1 # Solo consume electricidad de red".parse::<Components>().unwrap();
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
                cogen_to_grid: RenNrenCo2 {
                    ren: 0.0,
                    nren: 2.5,
                    co2: 0.3,
                },
                cogen_to_nepb: RenNrenCo2 {
                    ren: 0.0,
                    nren: 2.5,
                    co2: 0.3,
                },
            })
            .unwrap();
        let tfactors_normalized_stripped = tfactors_normalized.clone().strip(&tcomps);

        assert_eq!(tfactors_normalized.to_string(), tfactors_normalized_str);
        assert_eq!(tfactors_normalized_stripped.to_string(), tfactors_normalized_stripped_str);

    }
}
