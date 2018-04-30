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

use std::collections::HashMap;
use std::fmt;
use std::str;

use failure::Error;

use rennren::RenNren;

// == Common properties (carriers + weighting factors) ==

// Carrier name of an energy component or weighting factor
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display, EnumString)]
pub enum Carrier {
    ELECTRICIDAD,
    MEDIOAMBIENTE,
    BIOCARBURANTE,
    BIOMASA,
    BIOMASADENSIFICADA,
    CARBON,
    FUELOIL,
    GASNATURAL,
    GASOLEO,
    GLP,
    RED1,
    RED2,
}

// == Energy Components ==

// Produced or consumed energy type of an energy component
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString)]
pub enum CType {
    PRODUCCION,
    CONSUMO,
}

// Production origin or use destination subtype of an energy component
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Display, EnumString)]
pub enum CSubtype {
    INSITU,
    COGENERACION,
    EPB,
    NEPB,
}

// Destination Service or use of an energy component
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display)]
pub enum Service {
    ACS,
    CAL,
    REF,
    VEN,
    ILU,
    HU,
    DHU,
    NDEF,
}

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
            "ILU" => Ok(Service::ACS),
            "HU" => Ok(Service::ACS),
            "DHU" => Ok(Service::ACS),
            "" => Ok(Service::NDEF),
            "NDEF" => Ok(Service::NDEF),
            _ => Err(format_err!("Service not found")),
        }
    }
}

// == Weighting factors ==

// Source of energy for a weighting factor
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString)]
pub enum Source {
    RED,
    INSITU,
    COGENERACION,
}

// Destination of energy for a weighting factor
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString)]
pub enum Dest {
    input,
    to_grid,
    to_nEPB,
}

// Calculation step for a weighting factor
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString)]
pub enum Step {
    A,
    B,
}

// == General types ==

// Metadata Struct
// * objects of type 'META' represent metadata of components or weighting factors
//   - key is the metadata name
//   - value is the metadata value
#[derive(Debug, Clone)]
pub struct Meta {
    pub key: String,
    pub value: String,
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#META {}: {}", self.key, self.value)
    }
}

impl str::FromStr for Meta {
    type Err = Error;

    fn from_str(s: &str) -> Result<Meta, Self::Err> {
        // Remove start of line with #META or #CTE_
        let items: Vec<&str> = s.trim()[5..].splitn(2, ':').map(|v| v.trim()).collect();
        if items.len() == 2 {
            let key = items[0].to_string();
            let value = items[1].to_string();
            Ok(Meta { key, value })
        } else {
            Err(format_err!("Couldn't parse Metadata from string"))
        }
    }
}

// Energy Component Struct, representing an energy carrier component
//   - carrier is the carrier name
//   - ctype is either 'PRODUCCION' or 'CONSUMO' for produced or used energy
//   - csubtype defines:
//     - the energy origin for produced energy (INSITU or COGENERACION)
//     - the energy end use (EPB or NEPB) for delivered energy
//   - values is a list of energy values, one for each timestep
//   - comment is a comment string for the carrier
#[derive(Debug, Clone)]
pub struct Component {
    pub carrier: Carrier,
    pub ctype: CType,
    pub csubtype: CSubtype,
    pub service: Service,
    pub values: Vec<f32>,
    pub comment: String,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let valuelist = self.values
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
        let items: Vec<&str> = s.trim().splitn(2, '#').map(|v| v.trim()).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(|v| v.trim()).collect();
        if items.len() < 4 {
            return Err(format_err!(
                "Couldn't parse Component (Component) from string"
            ));
        };
        let carrier: Carrier = items[0].parse()?;
        let ctype: CType = items[1].parse()?;
        let csubtype: CSubtype = items[2].parse()?;
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

// Weighting Factor Struct
//
// It can represent the renewable and non renewable weighting factors,
// and is used for primary energy computation, but could also be used to get CO2 depending
// on how the values are obtained.
//
//   - carrier is the carrier name
//   - source is the origin that provides the carrier (RED, INSITU or COGENERACION)
//   - dest is the destination use of the energy (input, to_grid, to_nEPB)
//   - step is the evaluation step
//   - ren is the renewable amount for a unit of this carrier
//   - nren is the non renewable amount for a unit of this carrier
//   - comment is a comment string for the weighting factor
#[derive(Debug, Clone)]
pub struct Factor {
    pub carrier: Carrier,
    pub source: Source,
    pub dest: Dest,
    pub step: Step,
    pub ren: f32,
    pub nren: f32,
    pub comment: String,
}

impl Factor {
    pub fn factors(&self) -> RenNren {
        RenNren {
            ren: self.ren,
            nren: self.nren,
        }
    }
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        let items: Vec<&str> = s.trim().splitn(2, '#').map(|v| v.trim()).collect();
        let comment = items.get(1).unwrap_or(&"").to_string();
        let items: Vec<&str> = items[0].split(',').map(|v| v.trim()).collect();
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

// Components bundles a list of component data (Component) with its metadata (Meta)
//
// #META CTE_AREAREF: 100.5
// ELECTRICIDAD,CONSUMO,EPB,16.39,13.11,8.20,7.38,4.10,4.92,6.56,5.74,4.10,6.56,9.84,13.11
// ELECTRICIDAD,PRODUCCION,INSITU,8.20,6.56,4.10,3.69,2.05,2.46,3.28,2.87,2.05,3.28,4.92,6.56
// TODO: cmeta -> meta, cdata -> data
#[derive(Debug, Clone)]
pub struct Components {
    pub cmeta: Vec<Meta>,
    pub cdata: Vec<Component>,
}

impl fmt::Display for Components {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let metalines = self.cmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self.cdata
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
        let lines: Vec<&str> = s.lines().map(|v| v.trim()).collect();
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
            if &cdata_lens.iter().max().unwrap() != &cdata_lens.iter().min().unwrap() {
                return Err(format_err!(
                    "Energy components have different number of values: {:?}",
                    cdata_lens
                ));
            }
        }
        Ok(Components { cmeta, cdata })
    }
}

// Factors bundles a list of weighting factors (Factor) with its metadata (Meta)
// TODO: wmeta -> meta, wdata -> data
#[derive(Debug, Clone)]
pub struct Factors {
    pub wmeta: Vec<Meta>,
    pub wdata: Vec<Factor>,
}

impl fmt::Display for Factors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let metalines = self.wmeta
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>()
            .join("\n");
        let datalines = self.wdata
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
        let lines: Vec<&str> = s.lines().map(|v| v.trim()).collect();
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

// Results Struct for Output
//TODO: implement Display to serialize and FromStr to deserialize? JSON?

// BalanceForCarrier holds detailed results of the energy balance for a carrier
// TODO: add a carrier attribute to hold the carrier name.
#[derive(Debug, Clone)]
pub struct BalanceForCarrier {
    pub used_EPB: Vec<f32>,
    pub used_nEPB: Vec<f32>,
    pub produced_bygen: HashMap<CSubtype, Vec<f32>>, // es CSubtype pero es COGENERACION o INSITU (sourcetypes != RED)
    pub produced_bygen_an: HashMap<CSubtype, f32>,
    pub produced: Vec<f32>,
    pub produced_an: f32,
    pub f_match: Vec<f32>, // load matching factor
    // E_pr_cr_used_EPus_t <- produced_used_EPus
    // E_pr_cr_i_used_EPus_t <- produced_used_EPus_bygen
    pub exported: Vec<f32>, // exp_used_nEPus + exp_grid
    pub exported_an: f32,
    pub exported_bygen: HashMap<CSubtype, Vec<f32>>, // cambiado origin -> gen
    pub exported_bygen_an: HashMap<CSubtype, f32>,   // cambiado origin -> gen
    pub exported_grid: Vec<f32>,
    pub exported_grid_an: f32,
    pub exported_nEPB: Vec<f32>,
    pub exported_nEPB_an: f32,
    pub delivered_grid: Vec<f32>,
    pub delivered_grid_an: f32,
    // Weighted energy: { ren, nren }
    pub we_delivered_grid_an: RenNren,
    pub we_delivered_prod_an: RenNren,
    pub we_delivered_an: RenNren,
    pub we_exported_an_A: RenNren,
    pub we_exported_nEPB_an_AB: RenNren,
    pub we_exported_grid_an_AB: RenNren,
    pub we_exported_an_AB: RenNren,
    pub we_exported_an: RenNren,
    pub we_an_A: RenNren,
    pub we_an: RenNren,
}

// BalanceTotal holds global balance results, either in absolute value or by m2.
#[derive(Debug, Copy, Clone, Default)]
pub struct BalanceTotal {
    pub A: RenNren,
    pub B: RenNren,
    pub we_del: RenNren,
    pub we_exp_A: RenNren,
    pub we_exp: RenNren,
}

// Balance holds both the data and the results of an energy performance computation
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Balance {
    pub components: Components,
    pub wfactors: Factors,
    pub k_exp: f32,
    pub arearef: f32,
    pub balance_cr_i: HashMap<Carrier, BalanceForCarrier>,
    pub balance: BalanceTotal,
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
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0
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
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0
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
            dest: "input".parse().unwrap(),
            step: "A".parse().unwrap(),
            ren: 0.414,
            nren: 1.954,
            comment: "Electricidad de red paso A".into(),
        };
        let factor1str = "ELECTRICIDAD, RED, input, A, 0.414, 1.954 # Electricidad de red paso A";
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
ELECTRICIDAD, RED, input, A, 0.414, 1.954 # Recursos usados para suministrar electricidad (peninsular) desde la red
ELECTRICIDAD, INSITU, input, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ";

        // roundtrip building from/to string
        assert_eq!(
            format!("{}", tfactors1.parse::<Factors>().unwrap()),
            tfactors1
        );
    }
}
