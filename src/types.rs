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

// Author(s): Rafael Villar Burke <pachi@ietcc.csic.es>

//TODO: add produced_used_EPus to BalanceForCarrier (E_pr_cr_used_EPus_t)
//TODO: add produced_used_EPus_bygen to BalanceForCarrier (E_pr_cr_i_used_EPus_t)

use std::collections::HashMap;
use std::fmt;
use std::str;
use std::str::FromStr;

use failure::Error;

use crate::rennren::RenNren;

// == Common properties (carriers + weighting factors) ==

/// Energy carrier.
#[allow(non_camel_case_types)]
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, EnumString, Serialize,
)]
pub enum Carrier {
    /// Electricity
    ELECTRICIDAD,
    /// Environment thermal energy
    MEDIOAMBIENTE,
    /// Biofuel
    BIOCARBURANTE,
    /// Biomass
    BIOMASA,
    /// Densified biomass (pellets)
    BIOMASADENSIFICADA,
    /// Coal
    CARBON,
    /// Natural gas
    GASNATURAL,
    /// Diesel oil
    GASOLEO,
    /// LPG - Liquefied petroleum gas
    GLP,
    /// Generic energy carrier 1
    RED1,
    /// Generic energy carrier 2
    RED2,
}

// == Energy Components ==

/// Produced or consumed energy type of an energy component.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString, Serialize)]
pub enum CType {
    /// Produced energy
    PRODUCCION,
    /// Consumed energy
    CONSUMO,
}

/// Production origin or use destination subtype of an energy component.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Display, EnumString, Serialize)]
pub enum CSubtype {
    /// on site energy source
    INSITU,
    /// cogeneration energy source
    COGENERACION,
    /// EPB use
    EPB,
    /// Non EPB use
    NEPB,
}

/// Destination Service or use of an energy component.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Display, Serialize)]
pub enum Service {
    /// DHW
    ACS,
    /// Heating
    CAL,
    /// Cooling
    REF,
    /// Ventilation
    VEN,
    /// Lighting
    ILU,
    /// Humidification
    HU,
    /// Dehumidification
    DHU,
    /// Undefined or generic use
    NDEF,
    // TODO:
    // BAC
    // Building automation and control
}

pub const SERVICES: [Service; 8] = [
    Service::ACS,
    Service::CAL,
    Service::REF,
    Service::VEN,
    Service::ILU,
    Service::HU,
    Service::DHU,
    Service::NDEF,
    // BAC
];

impl str::FromStr for Service {
    type Err = Error;

    fn from_str(s: &str) -> Result<Service, Self::Err> {
        match s {
            "ACS" => Ok(Service::ACS),
            "WATERSYSTEMS" => Ok(Service::ACS),
            "CAL" => Ok(Service::CAL),
            "HEATING" => Ok(Service::CAL),
            "REF" => Ok(Service::REF),
            "COOLING" => Ok(Service::REF),
            "VEN" => Ok(Service::VEN),
            "FANS" => Ok(Service::VEN),
            "ILU" => Ok(Service::ILU),
            "HU" => Ok(Service::HU),
            "DHU" => Ok(Service::DHU),
            "" => Ok(Service::NDEF),
            "NDEF" => Ok(Service::NDEF),
            _ => Err(format_err!("Service not found")),
        }
    }
}

// == Weighting factors ==

/// Source of energy for a weighting factor.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString, Serialize)]
pub enum Source {
    /// Grid source
    RED,
    /// Insitu generation source
    INSITU,
    /// Cogeneration source
    COGENERACION,
}

/// Destination of energy for a weighting factor.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString, Serialize)]
pub enum Dest {
    /// Building delivery destination
    SUMINISTRO,
    /// Grid destination
    A_RED,
    /// Non EPB uses destination
    A_NEPB,
}

/// Calculation step for a weighting factor.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString, Serialize)]
pub enum Step {
    /// Calculation step A
    A,
    /// Calculation step B
    B,
}

// == General types ==

/// Metadata of components or weighting factors
#[derive(Debug, Clone, Serialize)]
pub struct Meta {
    /// metadata name.
    pub key: String,
    /// metadata value
    pub value: String,
}

impl fmt::Display for Meta {
    /// Textual representation of metadata.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#META {}: {}", self.key, self.value)
    }
}

impl str::FromStr for Meta {
    type Err = Error;

    fn from_str(s: &str) -> Result<Meta, Self::Err> {
        // Remove start of line with #META or #CTE_
        let items: Vec<&str> = s.trim()[5..].splitn(2, ':').map(str::trim).collect();
        if items.len() == 2 {
            let key = match items[0].trim() {
                // Fix legacy values
                "Localizacion" => "CTE_LOCALIZACION".to_string(),
                "Area_ref" => "CTE_AREAREF".to_string(),
                "kexp" => "CTE_KEXP".to_string(),
                x => x.to_string(),
            };
            let value = items[1].trim().to_string();
            Ok(Meta { key, value })
        } else {
            Err(format_err!("Couldn't parse Metadata from string"))
        }
    }
}

/// Energy Component Struct, representing an energy carrier component
#[derive(Debug, Clone, Serialize)]
pub struct Component {
    /// Carrier name
    pub carrier: Carrier,
    /// Produced (`PRODUCCION`) or consumed (`CONSUMO`) component type
    pub ctype: CType,
    /// Energy origin (`INSITU` or `COGENERACION`) for produced component types or end use type (`EPB` or `NEPB`) for consumed component types
    pub csubtype: CSubtype,
    /// End use
    pub service: Service,
    /// List of energy values, one value for each timestep
    pub values: Vec<f32>,
    /// Descriptive comment string
    pub comment: String,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let valuelist = self
            .values
            .iter()
            .map(|v| format!("{:.2}", v))
            .collect::<Vec<_>>()
            .join(", ");
        let comment = if self.comment != "" {
            format!(" # {}", self.comment)
        } else {
            "".to_owned()
        };
        write!(
            f,
            "{}, {}, {}, {}, {}{}",
            self.carrier, self.ctype, self.csubtype, self.service, valuelist, comment
        )
    }
}

impl str::FromStr for Component {
    type Err = Error;

    fn from_str(s: &str) -> Result<Component, Self::Err> {
        use self::CSubtype::*;
        use self::CType::*;
        use self::Carrier::{ELECTRICIDAD, MEDIOAMBIENTE};

        let items: Vec<&str> = s.trim().splitn(2, '#').map(str::trim).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(str::trim).collect();
        if items.len() < 4 {
            return Err(format_err!(
                "Couldn't parse Component (Component) from string: {}",
                s
            ));
        };
        let carrier: Carrier = items[0].parse()?;
        let ctype: CType = items[1].parse()?;
        let csubtype: CSubtype = items[2].parse()?;
        let carrier_ok = match ctype {
            CONSUMO => match csubtype {
                EPB | NEPB => true,
                _ => false,
            },
            PRODUCCION => match csubtype {
                INSITU => carrier == ELECTRICIDAD || carrier == MEDIOAMBIENTE,
                COGENERACION => carrier == ELECTRICIDAD,
                _ => false,
            },
        };
        if !carrier_ok {
            return Err(format_err!("Wrong Component definition in string: {}", s));
        }
        //This accounts for the legacy version, which may not have a service type
        let maybeservice: Result<Service, _> = items[3].parse();
        let (valuesidx, service) = match maybeservice {
            Ok(s) => (4, s),
            Err(_) => (3, Service::NDEF),
        };
        let values = items[valuesidx..]
            .iter()
            .map(|v| v.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;
        Ok(Component {
            carrier,
            ctype,
            csubtype,
            service,
            values,
            comment,
        })
    }
}

/// Weighting Factor Struct
///
/// It can represent the renewable and non renewable primary energy weighting factors,
/// but can be used for CO2 or any other indicators depending on how the values are obtained.
#[derive(Debug, Clone, Serialize)]
pub struct Factor {
    /// Energy carrier
    pub carrier: Carrier,
    /// Carrier source (`RED`, `INSITU` or `COGENERACION`)
    pub source: Source,
    /// Destination use of the energy (`SUMINISTRO`, `A_RED`, `A_NEPB`)
    pub dest: Dest,
    /// Evaluation step
    pub step: Step,
    /// Renewable primary energy for each end use unit of this carrier
    pub ren: f32,
    /// Non renewable primary energy for each end use unit of this carrier
    pub nren: f32,
    /// Descriptive comment string for the weighting factor
    pub comment: String,
}

impl Factor {
    /// Constructor
    pub fn new(
        carrier: Carrier,
        source: Source,
        dest: Dest,
        step: Step,
        ren: f32,
        nren: f32,
        comment: String,
    ) -> Factor {
        Factor {
            carrier,
            source,
            dest,
            step,
            ren,
            nren,
            comment,
        }
    }

    /// Get factors as RenNren struct
    pub fn factors(&self) -> RenNren {
        RenNren {
            ren: self.ren,
            nren: self.nren,
        }
    }
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let comment = if self.comment != "" {
            format!(" # {}", self.comment)
        } else {
            "".to_owned()
        };
        write!(
            f,
            "{}, {}, {}, {}, {:.3}, {:.3}{}",
            self.carrier, self.source, self.dest, self.step, self.ren, self.nren, comment
        )
    }
}

impl str::FromStr for Factor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Factor, Self::Err> {
        let items: Vec<&str> = s.trim().splitn(2, '#').map(str::trim).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(str::trim).collect();
        if items.len() < 6 {
            return Err(format_err!(
                "Couldn't parse Weighting Factor (Factor) from string"
            ));
        };
        let carrier: Carrier = items[0].parse()?;
        let source: Source = items[1].parse()?;
        let dest: Dest = items[2].parse()?;
        let step: Step = items[3].parse()?;
        let ren: f32 = items[4].parse()?;
        let nren: f32 = items[5].parse()?;
        Ok(Factor {
            carrier,
            source,
            dest,
            step,
            ren,
            nren,
            comment,
        })
    }
}

// == Data + Metadata Types ==

/// Common trait for handling metadata

pub trait MetaVec {
    fn get_metavec(&self) -> &Vec<Meta>;
    fn get_mut_metavec(&mut self) -> &mut Vec<Meta>;

    /// Check if key is included in metadata
    fn has_meta(&self, key: &str) -> bool {
        self.get_metavec().iter().any(|m| m.key == key)
    }

    /// Get (optional) metadata value by key
    fn get_meta(&self, key: &str) -> Option<String> {
        self.get_metavec()
            .iter()
            .find(|m| m.key == key)
            .and_then(|v| Some(v.value.clone()))
    }

    /// Get (optional) metadata value (f32) by key as f32
    fn get_meta_f32(&self, key: &str) -> Option<f32> {
        self.get_metavec()
            .iter()
            .find(|m| m.key == key)
            .and_then(|v| f32::from_str(v.value.trim()).ok())
    }

    /// Get (optional) metadata value (f32, f32) by key as RenNren struct
    fn get_meta_rennren(&self, key: &str) -> Option<RenNren> {
        self.get_metavec()
            .iter()
            .find(|m| m.key == key)
            .and_then(|v| {
                let vals = v
                    .value
                    .split(',')
                    .map(|s| f32::from_str(s.trim()).ok())
                    .collect::<Option<Vec<f32>>>()
                    .unwrap_or_else(|| {
                        panic!("No se puede transformar el metadato a RenNren: {:?}", v)
                    });
                if vals.len() != 2 {
                    None
                } else {
                    Some(RenNren {
                        ren: vals[0],
                        nren: vals[1],
                    })
                }
            })
    }

    /// Update metadata value for key or insert new metadata.
    fn update_meta(&mut self, key: &str, value: &str) {
        let val = value.to_string();
        let wmeta = self.get_mut_metavec();
        let metapos = wmeta.iter().position(|m| m.key == key);
        if let Some(pos) = metapos {
            wmeta[pos].value = val;
        } else {
            wmeta.push(Meta {
                key: key.to_string(),
                value: val,
            });
        };
    }
}

/// List of component data bundled with its metadata
///
/// #META CTE_AREAREF: 100.5
/// ELECTRICIDAD,CONSUMO,EPB,16.39,13.11,8.20,7.38,4.10,4.92,6.56,5.74,4.10,6.56,9.84,13.11
/// ELECTRICIDAD,PRODUCCION,INSITU,8.20,6.56,4.10,3.69,2.05,2.46,3.28,2.87,2.05,3.28,4.92,6.56
#[derive(Debug, Default, Clone, Serialize)]
pub struct Components {
    /// Component list
    pub cmeta: Vec<Meta>,
    /// Metadata
    pub cdata: Vec<Component>,
}

impl MetaVec for Components {
    fn get_metavec(&self) -> &Vec<Meta> {
        &self.cmeta
    }
    fn get_mut_metavec(&mut self) -> &mut Vec<Meta> {
        &mut self.cmeta
    }
}

impl fmt::Display for Components {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metalines = self
            .cmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self
            .cdata
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n{}", metalines, datalines)
    }
}

impl str::FromStr for Components {
    type Err = Error;

    fn from_str(s: &str) -> Result<Components, Self::Err> {
        let s_nobom = if s.starts_with("\u{feff}") {
            &s[3..]
        } else {
            s
        };
        let lines: Vec<&str> = s_nobom.lines().map(str::trim).collect();
        let metalines = lines
            .iter()
            .filter(|l| l.starts_with("#META") || l.starts_with("#CTE_"));
        let datalines = lines
            .iter()
            .filter(|l| !(l.starts_with('#') || l.starts_with("vector,") || l.is_empty()));
        let cmeta = metalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Meta>, _>>()?;
        let cdata = datalines
            .map(|e| e.parse())
            .collect::<Result<Vec<Component>, _>>()?;
        {
            let cdata_lens: Vec<_> = cdata.iter().map(|e| e.values.len()).collect();
            if cdata_lens.iter().max().unwrap() != cdata_lens.iter().min().unwrap() {
                return Err(format_err!(
                    "Energy components have different number of values: {:?}",
                    cdata_lens
                ));
            }
        }
        Ok(Components { cmeta, cdata })
    }
}

/// List of weighting factors bundled with its metadata
#[derive(Debug, Default, Clone, Serialize)]
pub struct Factors {
    /// Weighting factors list
    pub wmeta: Vec<Meta>,
    /// Metadata
    pub wdata: Vec<Factor>,
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
    type Err = Error;

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

/// Detailed results of the energy balance computation for a given carrier
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize)]
pub struct BalanceForCarrier {
    /// Energy carrier
    pub carrier: Carrier,
    /// Energy used for EPB uses in each timestep
    pub used_EPB: Vec<f32>,
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
    // TODO: add these:
    // - E_pr_cr_used_EPus_t <- produced_used_EPus
    // - E_pr_cr_i_used_EPus_t <- produced_used_EPus_bygen
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
    pub we_delivered_grid_an: RenNren,
    /// Weighted delivered energy by any energy production sources
    pub we_delivered_prod_an: RenNren,
    /// Weighted delivered energy by the grid and any energy production sources
    pub we_delivered_an: RenNren,
    /// Weighted exported energy for calculation step A
    pub we_exported_an_A: RenNren,
    /// Weighted exported energy for non EPB uses and calculation step AB
    pub we_exported_nEPB_an_AB: RenNren,
    /// Weighted exported energy to the grid and calculation step AB
    pub we_exported_grid_an_AB: RenNren,
    /// Weighted exported energy and calculation step AB
    pub we_exported_an_AB: RenNren,
    /// Weighted exported energy for calculation step A+B
    pub we_exported_an: RenNren,
    /// Weighted energy for calculation step A
    pub we_an_A: RenNren,
    /// Weighted energy for calculation step A, by use (for EPB services)
    pub we_an_A_byuse: HashMap<Service, RenNren>,
    /// Weighted energy
    pub we_an: RenNren,
    /// Weighted energy, by use (for EPB services)
    pub we_an_byuse: HashMap<Service, RenNren>,
}

/// Global balance results (all carriers), either in absolute value or by m2.
#[allow(non_snake_case)]
#[derive(Debug, Clone, Default, Serialize)]
pub struct BalanceTotal {
    /// Balance result for calculation step A
    pub A: RenNren,
    /// Weighted energy for calculation step A, by use (for EPB services)
    pub A_byuse: HashMap<Service, RenNren>,
    /// Balance result for calculation step A+B
    pub B: RenNren,
    /// Weighted energy, by use (for EPB services)
    pub B_byuse: HashMap<Service, RenNren>,
    /// Weighted delivered energy
    pub we_del: RenNren,
    /// Weighted exported energy for calculation step A
    pub we_exp_A: RenNren,
    /// Weighted exported energy for calculation step A+B
    pub we_exp: RenNren,
}

/// Data and results of an energy performance computation
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
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
    pub balance_cr_i: HashMap<Carrier, BalanceForCarrier>,
    /// Global energy balance results
    pub balance: BalanceTotal,
    /// Global energy balance results expressed as area ratios
    pub balance_m2: BalanceTotal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmeta() {
        let meta = Meta {
            key: "CTE_FUENTE".to_string(),
            value: "CTE2013".to_string(),
        };
        let metastr = "#META CTE_FUENTE: CTE2013";
        assert_eq!(format!("{}", meta), metastr);
        assert_eq!(format!("{}", metastr.parse::<Meta>().unwrap()), metastr);
    }

    #[test]
    fn tcomponent() {
        let component1 = Component {
            carrier: "ELECTRICIDAD".parse().unwrap(),
            ctype: "CONSUMO".parse().unwrap(),
            csubtype: "EPB".parse().unwrap(),
            service: "REF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario cons 1".into(),
        };
        let component1str = "ELECTRICIDAD, CONSUMO, EPB, REF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario cons 1";
        let component2 = Component {
            carrier: "ELECTRICIDAD".parse().unwrap(),
            ctype: "PRODUCCION".parse().unwrap(),
            csubtype: "INSITU".parse().unwrap(),
            service: "NDEF".parse().unwrap(),
            values: vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
            comment: "Comentario prod 1".into(),
        };
        let component2str = "ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";
        let component2strlegacy = "ELECTRICIDAD, PRODUCCION, INSITU, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";

        // consumer component
        assert_eq!(format!("{}", component1), component1str);

        // producer component
        assert_eq!(format!("{}", component2), component2str);

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", component2str.parse::<Component>().unwrap()),
            component2str
        );
        // roundtrip building from/to string for legacy format
        assert_eq!(
            format!("{}", component2strlegacy.parse::<Component>().unwrap()),
            component2str
        );
    }

    #[test]
    fn tfactor() {
        let factor1 = Factor {
            carrier: "ELECTRICIDAD".parse().unwrap(),
            source: "RED".parse().unwrap(),
            dest: "SUMINISTRO".parse().unwrap(),
            step: "A".parse().unwrap(),
            ren: 0.414,
            nren: 1.954,
            comment: "Electricidad de red paso A".into(),
        };
        let factor1str =
            "ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954 # Electricidad de red paso A";
        let factor2str = "ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 1.00, 2.00, 3.00, 4.00, 5.00, 6.00, 7.00, 8.00, 9.00, 10.00, 11.00, 12.00 # Comentario prod 1";

        // consumer component
        assert_eq!(format!("{}", factor1), factor1str);

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", factor2str.parse::<Component>().unwrap()),
            factor2str
        );
    }

    #[test]
    fn tcomponents() {
        let tcomponents1 = "#META CTE_AREAREF: 100.5
ELECTRICIDAD, CONSUMO, EPB, NDEF, 16.39, 13.11, 8.20, 7.38, 4.10, 4.92, 6.56, 5.74, 4.10, 6.56, 9.84, 13.11
ELECTRICIDAD, PRODUCCION, INSITU, NDEF, 8.20, 6.56, 4.10, 3.69, 2.05, 2.46, 3.28, 2.87, 2.05, 3.28, 4.92, 6.56";

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", tcomponents1.parse::<Components>().unwrap()),
            tcomponents1
        );
    }

    #[test]
    fn tfactors() {
        let tfactors1 = "#META CTE_FUENTE: CTE2013
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del IDAE de 20/07/2014
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ";

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", tfactors1.parse::<Factors>().unwrap()),
            tfactors1
        );
    }
}
