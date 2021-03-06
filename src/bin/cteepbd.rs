// Copyright (c) 2018-2020  Ministerio de Fomento
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
CteEPBD CLI app
===============

cteepbd - Implementation of the ISO EN 52000-1 standard
-------------------------------------------------------

  Energy performance of buildings - Overarching EPB assessment - General framework and procedures
  This implementation has used the following assumptions:
  - weighting factors are constant for all timesteps
  - no priority is set for energy production (average step A weighting factor f_we_el_stepA)
  - all on-site produced energy from non cogeneration sources is considered as delivered
  - on-site produced energy is not compensated on a service by service basis, but on a by carrier basis
  - the load matching factor is constant and equal to 1.0
  TODO:
  - allow other values of the load matching factor (or usign functions) f_match_t (formula 32, B.32)

*/

use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;

use cteepbd::{
    cte, energy_performance,
    types::{MetaVec, RenNrenCo2, Service},
    Balance, Components, UserWF,
};

const APP_TITLE: &str = r#"CteEPBD"#;
const APP_DESCRIPTION: &str = r#"
Copyright (c) 2018-2020 Ministerio de Fomento,
              Instituto de CC. de la Construcción Eduardo Torroja (IETcc-CSIC)

Autores: Rafael Villar Burke <pachi@ietcc.csic.es>,
         Daniel Jiménez González <danielj@ietcc.csic.es>
         Marta Sorribes Gil <msorribes@ietcc.csic.es>

Licencia: Publicado bajo licencia MIT.

"#;
const APP_ABOUT: &str = r#"CteEpbd - Eficiencia energética de los edificios (CTE DB-HE)."#;
const APP_LICENSE: &str = r#"
Copyright (c) 2018-2020 Ministerio de Fomento
              Instituto de Ciencias de la Construcción Eduardo Torroja (IETcc-CSIC)

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the 'Software'), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

Author(s): Rafael Villar Burke <pachi@ietcc.csic.es>
            Daniel Jiménez González <danielj@ietcc.csic.es>
            Marta Sorribes Gil <msorribes@ietcc.csic.es>"#;

// Funciones auxiliares -----------------------------------------------------------------------

fn readfile<P: AsRef<Path>>(path: P) -> String {
    read_to_string(&path).unwrap_or_else(|e| {
        eprintln!(
            "ERROR: lectura incorrecta del archivo \"{}\": {}",
            path.as_ref().display(),
            e
        );
        exit(exitcode::IOERR);
    })
}

fn writefile<P: AsRef<Path>>(path: P, content: &[u8]) {
    let mut file = File::create(&path)
        .map_err(|e| {
            eprintln!(
                "ERROR: no se ha podido crear el archivo \"{}\": {}",
                path.as_ref().display(),
                e
            );
            exit(exitcode::CANTCREAT);
        })
        .unwrap();
    if let Err(e) = file.write_all(content) {
        eprintln!(
            "ERROR: no se ha podido escribir en el archivo \"{}\": {}",
            path.as_ref().display(),
            e
        );
        exit(exitcode::IOERR);
    }
}

// Funciones auxiliares de validación y obtención de valores

/// Comprueba validez del valor del factor de exportación
fn validate_kexp(kexpstr: &str, orig: &str) -> Option<f32> {
    let kexp = kexpstr.parse::<f32>().unwrap_or_else(|_| {
        eprintln!(
            "ERROR: factor de exportación k_exp incorrecto \"{}\" ({})",
            kexpstr, orig
        );
        exit(exitcode::DATAERR);
    });
    if kexp < 0.0 || kexp > 1.0 {
        eprintln!(
            "ERROR: factor de exportación k_exp fuera de rango [0.00 - 1.00]: {:.2} ({})",
            kexp, orig
        );
        exit(exitcode::DATAERR);
    };
    if kexp != cte::KEXP_DEFAULT {
        println!(
            "AVISO: factor de exportación k_exp distinto al reglamentario ({:.2}): {:.2} ({})",
            cte::KEXP_DEFAULT,
            kexp,
            orig
        );
    };
    Some(kexp)
}

/// Comprueba validez del dato de area
fn validate_arearef(arearefstr: &str, orig: &str) -> Option<f32> {
    let arearef = arearefstr.parse::<f32>().unwrap_or_else(|_| {
        eprintln!(
            "ERROR: área de referencia A_ref incorrecta \"{}\" ({})",
            arearefstr, orig
        );
        exit(exitcode::DATAERR);
    });
    if arearef <= 1e-3 {
        eprintln!(
            "ERROR: área de referencia A_ref fuera de rango [0.001-]: {:.2} ({})",
            arearef, orig
        );
        exit(exitcode::DATAERR);
    }
    Some(arearef)
}

/// Obtiene factor de paso priorizando CLI -> metadatos -> None.
fn get_factor(
    matches: &clap::ArgMatches<'_>,
    components: &mut Components,
    meta: &str,
) -> Option<RenNrenCo2> {
    let factor = matches
        .values_of(meta)
        .and_then(|v| {
            // Datos desde línea de comandos
            let vv: Vec<f32> = v
                .map(|vv| {
                    f32::from_str(vv.trim()).unwrap_or_else(|_| {
                        eprintln!("ERROR: factor de paso incorrecto: \"{}\"", vv);
                        exit(exitcode::DATAERR);
                    })
                })
                .collect();
            Some(RenNrenCo2 {
                ren: vv[0],
                nren: vv[1],
                co2: vv[2],
            })
        })
        .or_else(|| components.get_meta_rennren(meta));
    if let Some(factor) = factor {
        components.set_meta(
            meta,
            &format!("{:.3}, {:.3}, {:.3}", factor.ren, factor.nren, factor.co2),
        );
    };
    factor
}

/// Carga componentes desde archivo o devuelve componentes por defecto
fn get_components(archivo: Option<&str>) -> Components {
    if let Some(archivo_componentes) = archivo {
        println!("Componentes energéticos: \"{}\"", archivo_componentes);
        readfile(archivo_componentes)
            .parse::<Components>()
            .unwrap_or_else(|e| {
                eprintln!(
                    "ERROR: formato incorrecto del archivo de componentes \"{}\": {}",
                    archivo_componentes, e
                );
                exit(exitcode::DATAERR);
            })
            .normalize()
    } else {
        Components::default()
    }
}

/// Crea aplicación y detecta opciones seleccionadas
fn start_app_and_get_matches() -> clap::ArgMatches<'static> {
    use clap::Arg;
    clap::App::new(APP_TITLE)
        .bin_name("cteepbd")
        .version(env!("CARGO_PKG_VERSION"))
        .author(APP_DESCRIPTION)
        .about(APP_ABOUT)
        .setting(clap::AppSettings::NextLineHelp)
        .arg(Arg::with_name("arearef")
            .short("a")
            .long("arearef")
            .value_name("AREAREF")
            .help("Área de referencia")
            .takes_value(true)
            .display_order(1))
        .arg(Arg::with_name("kexp")
            .short("k")
            .long("kexp")
            .value_name("KEXP")
            .help("Factor de exportación (k_exp)")
            .takes_value(true)
            .display_order(2))
        .arg(Arg::with_name("archivo_componentes")
            .short("c")
            .long("archivo_componentes")
            .value_name("ARCHIVO_COMPONENTES")
            .help("Archivo de definición de los componentes energéticos")
            .takes_value(true)
            //.validator(clap_validators::fs::is_file))
            .display_order(3))
        .arg(Arg::with_name("archivo_factores")
            .short("f")
            .long("archivo_factores")
            .value_name("ARCHIVO_FACTORES")
            .required_unless_one(&["fps_loc", "archivo_componentes"])
            .conflicts_with_all(&["fps_loc", "cogen", "red1", "red2"])
            .help("Archivo de definición de los componentes energéticos")
            .takes_value(true)
            //.validator(clap_validators::fs::is_file))
            .display_order(4))
        .arg(Arg::with_name("fps_loc")
            .short("l")
            .value_name("LOCALIZACION")
            .possible_values(&["PENINSULA", "CANARIAS", "BALEARES", "CEUTAMELILLA"])
            .required_unless_one(&["archivo_factores", "archivo_componentes"])
            .help("Localización que define los factores de paso\n")
            .takes_value(true)
            .display_order(5))
        // Archivos de salida
        .arg(Arg::with_name("gen_archivo_componentes")
            .long("oc")
            .value_name("GEN_ARCHIVO_COMPONENTES")
            .help("Archivo de salida de los vectores energéticos corregidos")
            .takes_value(true))
        .arg(Arg::with_name("gen_archivo_factores")
            .long("of")
            .value_name("GEN_ARCHIVO_FACTORES")
            .help("Archivo de salida de los factores de paso corregidos")
            .takes_value(true))
        .arg(Arg::with_name("archivo_salida_json")
            .long("json")
            .value_name("ARCHIVO_SALIDA_JSON")
            .help("Archivo de salida de resultados detallados en formato JSON")
            .takes_value(true))
        .arg(Arg::with_name("archivo_salida_xml")
            .long("xml")
            .value_name("ARCHIVO_SALIDA_XML")
            .help("Archivo de salida de resultados detallados en formato XML")
            .takes_value(true))
        .arg(Arg::with_name("archivo_salida_txt")
            .long("txt")
            .value_name("ARCHIVO_SALIDA_TXT")
            .help("Archivo de salida de resultados detallados en formato texto simple")
            .takes_value(true))
        // Factores definidos por el usuario
        .arg(Arg::with_name("CTE_RED1")
            .long("red1")
            .value_names(&["RED1_ren", "RED1_nren", "RED1_co2"])
            .help("Factores de paso (ren, nren, co2) de la producción del vector RED1.\nP.e.: --red1 0 1.3 0.3")
            .takes_value(true)
            .number_of_values(3))
        .arg(Arg::with_name("CTE_RED2")
            .long("red2")
            .value_names(&["RED2_ren", "RED2_nren", "RED2_co2"])
            .help("Factores de paso (ren, nren, co2) de la producción del vector RED2.\nP.e.: --red2 0 1.3 0.3")
            .takes_value(true)
            .number_of_values(3))
        .arg(Arg::with_name("CTE_COGEN")
            .long("cogen")
            .value_names(&["COGEN_ren", "COGEN_nren", "COGEN_co2"])
            .help("Factores de exportación a red (ren, nren, co2) de electricidad cogenerada.\nP.e.: --cogen 0 2.5 0.3")
            .takes_value(true)
            .number_of_values(3))
        .arg(Arg::with_name("CTE_COGENNEPB")
            .long("cogennepb")
            .value_names(&["COGENNEPB_ren", "COGENNEPB_nren", "COGENNEBP_co2"])
            .help("Factores de exportación a usos no EPB (ren, nren, co2) de electricidad cogenerada.\nP.e.: --cogennepb 0 2.5 0.3")
            .takes_value(true)
            .number_of_values(3))
        // Cálculo para servicio de ACS y factores en perímetro nearby
        .arg(Arg::with_name("demanda_anual_acs")
            .long("demanda_anual_acs")
            .value_name("DEM_ACS")
            .help("Demanda anual de ACS [kWh]"))
        .arg(Arg::with_name("acsnrb") // Obsoleto
            .short("N")
            .long("acs_nearby")
            .hidden(true)
            .requires("archivo_componentes")
            .help("Realiza el balance considerando solo el servicio de ACS y el perímetro nearby"))
        // Simplificación de factores
        .arg(Arg::with_name("nosimplificafps")
            .short("F")
            .long("no_simplifica_fps")
            .help("Evita la simplificación de los factores de paso según los vectores definidos"))
        // Opciones estándar: licencia y nivel de detalle
        .arg(Arg::with_name("showlicense")
            .short("L")
            .long("licencia")
            .help("Muestra la licencia del programa (MIT)"))
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .get_matches()
}

// Función principal ------------------------------------------------------------------------------

fn main() {
    let matches = start_app_and_get_matches();

    if matches.is_present("showlicense") {
        println!("{}", APP_LICENSE);
        exit(exitcode::OK);
    }

    // Prólogo ------------------------------------------------------------------------------------

    let verbosity = matches.occurrences_of("v");

    if verbosity > 2 {
        println!("Opciones indicadas: ----------");
        println!("{:#?}", matches);
        println!("------------------------------");
    }

    println!("** Datos de entrada");

    // Componentes energéticos ---------------------------------------------------------------------
    let mut components = get_components(matches.value_of("archivo_componentes"));

    // Cálculo para servicio de ACS en nearby
    if matches.is_present("acsnrb") {
        components = components.filter_by_epb_service(Service::ACS);
    }

    if verbosity > 1 && !components.cmeta.is_empty() {
        println!("Metadatos de componentes:");
        for meta in &components.cmeta {
            println!("  {}: {}", meta.key, meta.value);
        }
    }

    // Comprobación del parámetro de factor de exportación kexp -----------------------------------
    let kexp_cli = matches
        .value_of("kexp")
        .and_then(|kexpstr| validate_kexp(kexpstr, "usuario"));

    // Comprobación del parámetro de área de referencia -------------------------------------------
    let arearef_cli = matches
        .value_of("arearef")
        .and_then(|arearefstr| validate_arearef(arearefstr, "usuario"));

    // Factores de paso ---------------------------------------------------------------------------

    // 0. Factores por defecto, según modo
    let default_locwf = &cte::CTE_LOCWF_RITE2014;
    let default_userwf = cte::CTE_USERWF;

    // 1. Factores de paso definibles por el usuario (a través de la CLI o de metadatos)
    let user_wf = UserWF {
        red1: get_factor(&matches, &mut components, "CTE_RED1"),
        red2: get_factor(&matches, &mut components, "CTE_RED2"),
        cogen_to_grid: get_factor(&matches, &mut components, "CTE_COGEN"),
        cogen_to_nepb: get_factor(&matches, &mut components, "CTE_COGENNEPB"),
    };

    if verbosity > 2 {
        println!("Factores de paso de usuario:\n{:?}", user_wf)
    };

    // 2. Definición de los factores de paso principales

    let fp_path_cli = matches.value_of("archivo_factores");
    let loc_cli = matches.value_of("fps_loc");
    let loc_meta = components.get_meta("CTE_LOCALIZACION");

    // CLI path > CLI loc > Meta loc > error
    let (orig_fp, param_fp, fp_opt) = match (fp_path_cli, loc_cli, loc_meta) {
        (Some(fp_cli), _, _) => {
            let fp = cte::wfactors_from_str(&readfile(fp_cli), user_wf, default_userwf);
            ("archivo", fp_cli.to_string(), fp)
        }
        (None, Some(l_cli), _) => {
            let fp = cte::wfactors_from_loc(&l_cli, &default_locwf, user_wf, default_userwf);
            ("usuario", l_cli.to_string(), fp)
        }
        (None, None, Some(l_meta)) => {
            let fp = cte::wfactors_from_loc(&l_meta, &default_locwf, user_wf, default_userwf);
            ("metadatos", l_meta, fp)
        }
        _ => {
            eprintln!("ERROR: datos insuficientes para determinar los factores de paso");
            exit(exitcode::USAGE);
        }
    };

    let mut fpdata = fp_opt.unwrap_or_else(|e| {
        eprintln!(
            "ERROR: parámetros incorrectos para generar los factores de paso: {}",
            e
        );
        exit(exitcode::DATAERR);
    });

    println!("Factores de paso ({}): {}", orig_fp, param_fp);

    // Simplificación de los factores de paso -----------------------------------------------------
    if !matches.is_present("nosimplificafps") && !components.cdata.is_empty() {
        let oldfplen = fpdata.wdata.len();
        fpdata = fpdata.strip(&components);
        if verbosity > 1 {
            println!(
                "Reducción de factores de paso: {} a {}",
                oldfplen,
                fpdata.wdata.len()
            );
        }
    }

    // Transformación a factores de paso en nearby
    if matches.is_present("acsnrb") {
        // Estamos en cálculo de ACS en nearby
        fpdata = cte::wfactors_to_nearby(&fpdata);
    }

    // Área de referencia -------------------------------------------------------------------------
    // CLI > Metadatos de componentes > Valor por defecto (AREA_REF = 1)
    let arearef_meta = components
        .get_meta("CTE_AREAREF")
        .and_then(|ref arearefstr| validate_arearef(arearefstr, "metadatos"));

    if let (Some(a_meta), Some(a_cli)) = (arearef_meta, arearef_cli) {
        if (a_meta - a_cli).abs() > 1e-3 {
            println!("AVISO: área de referencia A_ref en componentes ({:.1}) y de usuario ({:.1}) distintos", a_meta, a_cli);
        };
    }

    // CLI > Meta > default
    let (orig_arearef, arearef) = match (arearef_meta, arearef_cli) {
        (_, Some(a_cli)) => ("usuario", a_cli),
        (Some(a_meta), _) => ("metadatos", a_meta),
        _ => ("predefinido", cte::AREAREF_DEFAULT),
    };

    // Actualiza metadato CTE_AREAREF al valor seleccionado
    components.set_meta("CTE_AREAREF", &format!("{:.2}", arearef));

    println!("Área de referencia ({}) [m2]: {:.2}", orig_arearef, arearef);

    // kexp ---------------------------------------------------------------------------------------
    // CLI > Metadatos de componentes > Valor por defecto (KEXP_REF = 0.0)
    let kexp_meta = components
        .get_meta("CTE_KEXP")
        .and_then(|ref kexpstr| validate_kexp(kexpstr, "metadatos"));

    if let (Some(k_meta), Some(k_cli)) = (kexp_meta, kexp_cli) {
        if (k_meta - k_cli).abs() > 1e-3 {
            println!("AVISO: factor de exportación k_exp en componentes ({:.1}) y de usuario ({:.1}) distintos", k_meta, k_cli);
        };
    }

    // CLI > Meta > default
    let (orig_kexp, kexp) = match (kexp_meta, kexp_cli) {
        (_, Some(k_cli)) => ("usuario", k_cli),
        (Some(k_meta), None) => ("metadatos", k_meta),
        _ => ("predefinido", cte::KEXP_DEFAULT),
    };

    // Actualiza metadato CTE_KEXP al valor seleccionado
    components.set_meta("CTE_KEXP", &format!("{:.1}", kexp));

    println!("Factor de exportación ({}) [-]: {:.1}", orig_kexp, kexp);

    // Guardado de componentes energéticos --------------------------------------------------------
    if matches.is_present("gen_archivo_componentes") {
        let path = matches.value_of_os("gen_archivo_componentes").unwrap();
        if verbosity > 2 {
            println!("Componentes energéticos:\n{}", components);
        }
        writefile(&path, components.to_string().as_bytes());
        if verbosity > 0 {
            println!("Guardado archivo de componentes energéticos: {:?}", path);
        }
    }

    // Guardado de factores de paso corregidos ----------------------------------------------------
    if matches.is_present("gen_archivo_factores") {
        let path = matches.value_of_os("gen_archivo_factores").unwrap();
        if verbosity > 2 {
            println!("Factores de paso:\n{}", fpdata);
        }
        writefile(&path, fpdata.to_string().as_bytes());
        if verbosity > 0 {
            println!("Guardado archivo de factores de paso: {:?}", path);
        }
    }

    // Demanda anual de ACS: CLI > Meta > None ----------------------------------------------------
    let maybe_demanda_anual_acs = matches
        .value_of("demanda_anual_acs")
        .and_then(|v| {
            v.parse::<f32>().ok().or_else(|| {
                eprintln!("ERROR: demanda anual de ACS con formato incorrecto");
                exit(exitcode::DATAERR);
            })
        })
        .or_else(|| components.get_meta_f32("CTE_ACS_DEMANDA_ANUAL"))
        .or(None);

    // Cálculo del balance ------------------------------------------------------------------------
    let balance: Option<Balance> = if !components.cdata.is_empty() {
        let balance = energy_performance(&components, &fpdata, kexp, arearef)
            .map(|b| cte::incorpora_demanda_renovable_acs_nrb(b, maybe_demanda_anual_acs))
            .unwrap_or_else(|e| {
                eprintln!(
                    "ERROR: no se ha podido calcular el balance energético: {}",
                    e
                );
                exit(exitcode::DATAERR);
            });
        Some(balance)
    } else if matches.is_present("gen_archivos_factores") {
        println!(
            "No se calcula el balance pero se ha generado el archivo de factores de paso {:?}",
            matches.value_of_os("gen_archivo_factores").unwrap()
        );
        None
    } else {
        println!("No se han definido datos suficientes para calcular el balance energético. Necesita definir al menos los componentes energéticos y los factores de paso");
        None
    };

    // Salida de resultados -----------------------------------------------------------------------
    if let Some(balance) = balance {
        // Guardar balance en formato json
        if matches.is_present("archivo_salida_json") {
            let path = matches.value_of_os("archivo_salida_json").unwrap();
            if verbosity > 0 {
                println!("Resultados en formato JSON: {:?}", path);
            }
            let json = serde_json::to_string_pretty(&balance).unwrap_or_else(|e| {
                eprintln!(
                    "ERROR: conversión incorrecta del balance energético a JSON: {}",
                    e
                );
                exit(exitcode::DATAERR);
            });
            writefile(&path, json.as_bytes());
        }
        // Guardar balance en formato XML
        if matches.is_present("archivo_salida_xml") {
            let path = matches.value_of_os("archivo_salida_xml").unwrap();
            if verbosity > 0 {
                println!("Resultados en formato XML: {:?}", path);
            }
            let xml = cte::balance_to_xml(&balance);
            writefile(&path, xml.as_bytes());
        }
        // Mostrar siempre en formato de texto plano
        if matches.is_present("acsnrb") {
            println!("** Balance energético (servicio de ACS, perímetro próximo)");
        } else {
            println!("** Balance energético");
        }
        let plain = cte::balance_to_plain(&balance);
        println!("{}", plain);

        // Guardar balance en formato de texto plano
        if matches.is_present("archivo_salida_txt") {
            let path = matches.value_of_os("archivo_salida_txt").unwrap();
            if verbosity > 0 {
                println!("Resultados en formato XML: {:?}", path);
            }
            writefile(&path, plain.as_bytes());
        }
    };
}
