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
//            Daniel Jiménez González <danielj@ietcc.csic.es>
//            Marta Sorribes Gil <msorribes@ietcc.csic.es>

#[macro_use]
extern crate clap;
extern crate epbdrs;
extern crate exitcode;
extern crate failure;
extern crate serde_json;

use std::fs::File;
use std::io::prelude::*;
//use std::path::PathBuf;
use std::path::Path;
use std::process;
use std::str::FromStr;

use clap::{App, Arg};
use failure::Fail;
use failure::Error;
use failure::ResultExt;

use epbdrs::cte;
use epbdrs::epbd::*;
use epbdrs::rennren::RenNren;
use epbdrs::types::{Balance, MetaVec, Service};

// Funciones auxiliares -----------------------------------------------------------------------

fn readfile(path: &Path) -> Result<String, Error> {
    let mut f = File::open(path).context(format!("Archivo {} no encontrado", path.display()))?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .context("Error al leer el archivo")?;
    Ok(contents)
}

fn writefile(path: &Path, content: &[u8]) {
    let mut file = match File::create(&path) {
        Err(err) => panic!("ERROR: no se ha podido escribir en \"{}\": {:?}",
                        path.display(), err.cause()),
        Ok(file) => file,
    };
    if let Err(err) = file.write_all(content) {
        panic!("No se ha podido escribir en {}: {:?}", path.display(), err.cause())
    }
}

fn main() {
    let matches = App::new("CteEPBD")
        .bin_name("cteepbd")
        .version(env!("CARGO_PKG_VERSION"))
        .author("
Copyright (c) 2018 Ministerio de Fomento,
                   Instituto de CC. de la Construcción Eduardo Torroja (IETcc-CSIC)

Autores: Rafael Villar Burke <pachi@ietcc.csic.es>,
         Daniel Jiménez González <danielj@ietcc.csic.es>
         Marta Sorribes Gil <msorribes@ietcc.csic.es>

Licencia: Publicado bajo licencia MIT.

")
        .about("CteEpbd - Eficiencia energética de los edificios (CTE DB-HE).")
        .arg(Arg::with_name("arearef")
            .short("a")
            .value_name("AREAREF")
            .default_value("1.0")
            .help("Área de referencia")
            .takes_value(true)
            .display_order(1))
        .arg(Arg::with_name("kexp")
            .short("k")
            .default_value("0.0")
            .value_name("KEXP")
            .help("Factor de exportación (k_exp)")
            .takes_value(true)
            .display_order(2))
        .arg(Arg::with_name("archivo_componentes")
            .short("c")
            .value_name("ARCHIVO_COMPONENTES")
            .help("Archivo de definición de los componentes energéticos")
            .takes_value(true)
            //.validator(clap_validators::fs::is_file))
            .display_order(3))
        .arg(Arg::with_name("archivo_factores")
            .short("f")
            .value_name("ARCHIVO_FACTORES")
            .required_unless("fps_loc")
            .conflicts_with_all(&["fps_loc", "cogen", "red1", "red2"])
            .help("Archivo de definición de los componentes energéticos")
            .takes_value(true)
            //.validator(clap_validators::fs::is_file))
            .display_order(3))
        .arg(Arg::with_name("fps_loc")
            .short("l")
            .value_name("LOCALIZACION")
            .possible_values(&["PENINSULA", "CANARIAS", "BALEARES", "CEUTAYMELILLA"])
            .required_unless("archivo_factores")
            .help("Localización que define los factores de paso")
            .takes_value(true)
            .display_order(4))
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
        .arg(Arg::with_name("cogen")
            .long("cogen")
            .value_names(&["COGEN_ren", "COGEN_nren"])
            .help("Factores de exportación a la red (ren, nren) de la electricidad cogenerada. P.e.: --cogen 0 2.5")
            .takes_value(true)
            .number_of_values(2))
        .arg(Arg::with_name("red1")
            .long("red1")
            .value_names(&["RED1_ren", "RED1_nren"])
            .help("Factores de paso (ren, nren) de la producción del vector RED1. P.e.: --red1 0 1.3")
            .takes_value(true)
            .number_of_values(2))
        .arg(Arg::with_name("red2")
            .long("red2")
            .value_names(&["RED2_ren", "RED2_nren"])
            .help("Factores de paso (ren, nren) de la producción del vector RED2. P.e.: --red2 0 1.3")
            .takes_value(true)
            .number_of_values(2))
        .arg(Arg::with_name("acsnrb")
            .short("N")
            .long("acs_nearby")
            .requires("archivo_componentes")
            .help("Realiza el balance considerando solo los componentes del servicio de ACS y el perímetro nearby"))
        .arg(Arg::with_name("nosimplificafps")
            .short("F")
            .long("no_simplifica_fps")
            .help("Evita la simplificación de los factores de paso en función de los vectores definidos"))
        .arg(Arg::with_name("showlicense")
            .short("L")
            .long("licencia")
            .help("Muestra la licencia del programa (MIT)"))
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .get_matches();

    if matches.is_present("showlicense") {
        println!(
            "
Copyright (c) 2018 Ministerio de Fomento
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
            Marta Sorribes Gil <msorribes@ietcc.csic.es>"
        );
        process::exit(exitcode::OK);
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
    let components = if let Some(archivo_componentes) = matches.value_of("archivo_componentes")
    {
        let path = Path::new(archivo_componentes);
        let componentsstring = match readfile(path) {
            Ok(componentsstring) => {
                println!("Componentes energéticos: \"{}\"", path.display());
                componentsstring
            },
            Err(err) => {
                eprintln!(
                    "ERROR: No se ha podido leer el archivo de componentes energéticos \"{}\" -> {}",
                    path.display(), err.cause()
                );
                process::exit(exitcode::IOERR);
            }
        };
        match cte::parse_components(&componentsstring) {
            Ok(components) => {
                // Estamos en cálculo de ACS en nearby
                if matches.is_present("acsnrb") {
                    Some(cte::components_by_service(&components, Service::ACS))
                } else {
                    Some(components)
                }
            },
            Err(err) => {
                eprintln!(
                    "ERROR: Formato incorrecto del archivo de componentes \"{}\" -> {}",
                    archivo_componentes, err.cause()
                );
                process::exit(exitcode::DATAERR);
            }
        }
    } else {
        None
    };
    let components_is_some = components.is_some(); // Marcador de que hay datos disponibles

    let mut components = if components_is_some {
        components.unwrap()
    } else {
        // Componentes vacío
        Default::default()
    };

    if verbosity > 1 && components_is_some {
        println!("Metadatos de componentes:");
        for meta in &components.cmeta {
            println!("  {}: {}", meta.key, meta.value);
        }
    }

    // Comprobación del parámetro de factor de exportación kexp ----------------------------------------
    if matches.is_present("kexp") {
        let kexp = value_t!(matches, "kexp", f32).unwrap_or_else(|error| {
            println!("El área de referencia indicado no es un valor numérico válido");
            if verbosity > 2 {
                println!("{}", error)
            };
            process::exit(exitcode::DATAERR);
        });
        if kexp < 0.0 || kexp > 1.0 {
            eprintln!(
                "ERROR: el factor de exportación debe estar entre 0.00 y 1.00 y vale {:.2}",
                kexp
            );
            process::exit(exitcode::DATAERR);
        };
        if kexp != cte::KEXP_DEFAULT {
            println!(
                "AVISO: factor de exportación k_exp ({:.2}) distinto al reglamentario ({:.2})",
                kexp,
                cte::KEXP_DEFAULT
            );
        };
    }

    // Comprobación del parámetro de área de referencia -------------------------------------------------------------------------
    if matches.is_present("arearef") {
        let arearef = value_t!(matches, "arearef", f32).unwrap_or_else(|error| {
            println!("El área de referencia indicado no es un valor numérico válido");
            if verbosity > 2 {
                println!("{}", error)
            };
            process::exit(exitcode::DATAERR);
        });
        if arearef <= 1e-3 {
            eprintln!("ERROR: el área de referencia definida por el usuario debe ser mayor que 0.00 y vale {:.2}", arearef);
            process::exit(exitcode::DATAERR);
        }
    }

    // Factores de paso ---------------------------------------------------------------------------

    // 1. Localización de los factores de paso genéricos y redefinibles a través de la CLI
    // 1.1 Coeficiente de paso de vector genérico 1 - RED1
    let red1 = matches
        .values_of("red1")
        .and_then(|v| {
            let vv: Vec<f32> = v.map(|vv| f32::from_str(vv.trim()).unwrap()).collect();
            let red1 = RenNren {
                ren: vv[0],
                nren: vv[1],
            };
            if verbosity > 2 {
                println!("Factores de paso para RED1 (usuario): {}", red1)};
            if components_is_some {
                components.update_meta("CTE_RED1", &format!("{:.3}, {:.3}", red1.ren, red1.nren));
            }
            Some(red1)
        })
        .or_else(|| {
            if !components_is_some {
                None
            } else {
                match components.get_meta_rennren("CTE_RED1") {
                    Some(red1) => {
                        if verbosity > 2 {
                        println!("Factores de paso para RED1 (metadatos): {}", red1);}
                        Some(red1)
                    }
                    _ => None,
                }
            }
        })
        .or_else(|| {
            let red1 = cte::CTE_RED_DEFAULTS_RED1;
            if verbosity > 2 {
            println!("Factores de paso para RED1 (predefinido): {}", red1);}
            Some(red1)
        });

    // 1.2 Coeficiente de paso de vector genérico 2 - RED2
    let red2 = matches
        .values_of("red2")
        .and_then(|v| {
            let vv: Vec<f32> = v.map(|vv| f32::from_str(vv.trim()).unwrap()).collect();
            let red2 = RenNren {
                ren: vv[0],
                nren: vv[1],
            };
            if verbosity > 2 { println!("Factores de paso para RED2 (usuario): {}", red2 );}
            if components_is_some {
                components.update_meta("CTE_RED2", &format!("{:.3}, {:.3}", red2.ren, red2.nren));
            }
            Some(red2)
        })
        .or_else(|| {
            if !components_is_some {
                None
            } else {
                match components.get_meta_rennren("CTE_RED2") {
                    Some(red2) => {
                        if verbosity > 2 {
                        println!("Factores de paso para RED2 (metadatos): {}",
                            red2
                        );}
                        Some(red2)
                    }
                    _ => None,
                }
            }
        })
        .or_else(|| {
            let red2 = cte::CTE_RED_DEFAULTS_RED2;
            if verbosity > 2 { println!(
                "Factores de paso para RED2 (predefinido): {}",
                red2
            );}
            Some(red2)
        });

    // 1.3 Coeficiente de paso de cogeneración a la red - COGEN
    let cogen = matches
        .values_of("cogen")
        .and_then(|v| {
            let vv: Vec<f32> = v.map(|vv| f32::from_str(vv.trim()).unwrap()).collect();
            let cogen = RenNren {
                ren: vv[0],
                nren: vv[1],
            };
            if verbosity > 2 {
            println!(
                "Factores de paso para COGENERACION a la red (usuario): {}",
                cogen
            );}
            if components_is_some {
                components
                    .update_meta("CTE_COGEN", &format!("{:.3}, {:.3}", cogen.ren, cogen.nren));
            }
            Some(cogen)
        })
        .or_else(|| {
            if !components_is_some {
                None
            } else {
                match components.get_meta_rennren("CTE_COGEN") {
                    Some(cogen) => {
                        if verbosity > 2 { println!(
                            "Factores de paso para COGENERACION a la red (metadatos): {}",
                            cogen
                        );}
                        Some(cogen)
                    }
                    _ => None,
                }
            }
        })
        .or_else(|| {
            let cogen = cte::CTE_COGEN_DEFAULTS_TO_GRID;
            if verbosity > 2 { println!(
                "Factores de paso para COGENERACION a la red (predefinido): {}",
                cogen
            );}
            Some(cogen)
        });

    // 1.4 Coeficiente de paso de cogeneración a usos no EPB - COGENNEPB
    let cogennepb = matches
        .values_of("cogennepb")
        .and_then(|v| {
            let vv: Vec<f32> = v.map(|vv| f32::from_str(vv.trim()).unwrap()).collect();
            let cogen = RenNren {
                ren: vv[0],
                nren: vv[1],
            };
            if verbosity > 2 { println!(
                "Factores de paso para COGENERACION a usos no EPB (usuario): {}",
                cogen
            );}
            if components_is_some {
                components.update_meta(
                    "CTE_COGENNEPB",
                    &format!("{:.3}, {:.3}", cogen.ren, cogen.nren),
                );
            }
            Some(cogen)
        })
        .or_else(|| {
            if !components_is_some {
                None
            } else {
                match components.get_meta_rennren("CTE_COGENNEPB") {
                    Some(cogen) => {
                        if verbosity > 2 { println!("Factores de paso para COGENERACION a usos no EPB (metadatos): {}", cogen);}
                        Some(cogen)
                    }
                    _ => None,
                }
            }
        })
        .or_else(|| {
            let cogen = cte::CTE_COGEN_DEFAULTS_TO_NEPB;
            if verbosity > 2 {println!(
                "Factores de paso para COGENERACION a usos no EPB (predefinido): {}",
                cogen
            );}
            Some(cogen)
        });

    // 2. Definición de los factores de paso principales
    let mut fpdata =
        // Definición desde archivo
        if let Some(archivo_factores) = matches.value_of("archivo_factores") {
            let path = Path::new(archivo_factores);
            let fpstring = match readfile(path) {
                Ok(fpstring) => {
                    println!("Factores de paso (archivo): \"{}\"", path.display());
                    fpstring
                },
                Err(err) => {
                    eprintln!(
                        "ERROR: No se ha podido leer el archivo de factores de paso \"{}\" -> {}",
                        path.display(), err.cause()
                    );
                    process::exit(exitcode::IOERR);
                }
            };
            cte::parse_wfactors(&fpstring, cogen, cogennepb, red1, red2, true)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "ERROR: No se ha podido interpretar el archivo de factores de paso \"{}\" -> {}",
                        path.display(), error.cause()
                    );
                    if verbosity > 2 { println!("{}", error.backtrace()) };
                    process::exit(exitcode::DATAERR);
                })
        // Definición por localización
        } else {
            let localizacion = matches
                .value_of("fps_loc")
                .and_then(|v| {
                    println!("Factores de paso (usuario): {}", v);
                    if components_is_some {
                        components.update_meta("CTE_LOCALIZACION", v);
                    }
                    Some(v.to_string())
                })
                .or_else(|| {
                    if !components_is_some {
                        None
                    } else {
                        match components.get_meta("CTE_LOCALIZACION") {
                        Some(loc) => {
                            println!("Factores de paso (metadatos): {}", loc);
                            Some(loc) },
                        _ => None
                        }
                    }
                })
                .or_else(|| {
                    eprintln!("ERROR: Sin datos suficientes para determinar los factores de paso");
                    process::exit(exitcode::USAGE);
                }).unwrap();

            cte::new_wfactors(&localizacion, cogen, cogennepb, red1, red2, false)
                .unwrap_or_else(|error| {
                    println!("ERROR: No se han podido generar los factores de paso");
                    if verbosity > 2 { println!("{}, {}", error.cause(), error.backtrace()) };
                    process::exit(exitcode::DATAERR);
                })
        };

    // Simplificación de los factores de paso -----------------------------------------------------------------
    if components_is_some && !matches.is_present("nosimplificafps") {
        let oldfplen = fpdata.wdata.len();
        cte::strip_wfactors(&mut fpdata, &components);
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
    // Orden de prioridad:
    // - Valor explícito en argumentos de CLI
    // - Valor definido en metadatos de componentes
    // - Valor por defecto (AREA_REF = 1)

    let mut arearef;
    // Se define CTE_AREAREF en metadatos de componentes energéticos
    if components_is_some && components.has_meta("CTE_AREAREF") {

        arearef = components.get_meta_f32("CTE_AREAREF").unwrap_or_else(|| {
            println!("El área de referencia de los metadatos no es un valor numérico válido");
            process::exit(exitcode::DATAERR);
        });
        if matches.occurrences_of("arearef") == 0 {
            println!("Área de referencia (metadatos) [m2]: {:.2}", arearef);
        } else {
            let m_arearef = value_t!(matches, "arearef", f32).unwrap();
            if (arearef - m_arearef).abs() > 1e-3 {
                println!("AVISO: El valor del área de referencia del archivo de componentes energéticos ({:.2}) no coincide con el valor definido por el usuario ({:.2})", arearef, m_arearef);
            }
            arearef = m_arearef;
            println!("Área de referencia (usuario) [m2]: {:.2}", arearef);
        }
    // Área de referencia en la interfaz
    } else if matches.occurrences_of("arearef") != 0 {
        arearef = value_t!(matches, "arearef", f32).unwrap();
        println!("Área de referencia (usuario) [m2]: {:.2}", arearef);
    // Valor por defecto
    } else {
        arearef = cte::AREAREF_DEFAULT;
        println!("Área de referencia (predefinida) [m2]: {:.2}", arearef);
    }

    // Actualiza metadato CTE_AREAREF al valor seleccionado
    if components_is_some {
        components.update_meta("CTE_AREAREF", &format!("{:.2}", arearef));
    }

    // kexp ------------------------------------------------------------------------------------------
    // Orden de prioridad:
    // - Valor explícito en argumentos de CLI
    // - Valor definido en metadatos de componentes
    // - Valor por defecto (KEXP_REF = 0.0)

    let mut kexp;
    // Se define CTE_KEXP en metadatos de componentes energéticos
    if components_is_some && components.has_meta("CTE_KEXP") {
        kexp = components.get_meta_f32("CTE_KEXP").unwrap_or_else(|| {
            println!("El factor de exportación de los metadatos no es un valor numérico válido");
            process::exit(exitcode::DATAERR);
        });
        if matches.occurrences_of("kexp") == 0 {
            println!("Factor de exportación (metadatos) [-]: {:.1}", kexp);
        } else {
            let m_kexp = value_t!(matches, "kexp", f32).unwrap();
            if (kexp - m_kexp).abs() > 1e-3 {
                println!("AVISO: El valor del factor de exportación del archivo de componentes energéticos ({:.1}) no coincide con el valor definido por el usuario ({:.1})", kexp, m_kexp);
            }
            kexp = m_kexp;
            println!("Factor de exportación (usuario) [-]: {:.1}", kexp);
        }
    // kexp definido en la interfaz
    } else if matches.occurrences_of("kexp") != 0 {
        kexp = value_t!(matches, "kexp", f32).unwrap();
        println!("Factor de exportación (usuario) [-]: {:.1}", kexp);
    // Valor por defecto
    } else {
        kexp = cte::KEXP_DEFAULT;
        println!("Factor de exportación (predefinido) [-]: {:.1}", kexp);
    }
    // Actualiza metadato CTE_KEXP al valor seleccionado
    if components_is_some {
        components.update_meta("CTE_KEXP", &format!("{:.1}", kexp));
    }

    // Guardado de componentes energéticos -----------------------------------------------------------
    if matches.is_present("gen_archivo_componentes") {
        let path = Path::new(matches.value_of_os("gen_archivo_componentes").unwrap());
        let components_string = format!("{}", components);
        if verbosity > 2 {
            println!("Componentes energéticos:\n{}", components_string);
        }
        writefile(&path, components_string.as_bytes());
        if verbosity > 0 {
            println!("Guardado archivo de componentes energéticos: {}", path.display());
        }
    }

    // Guardado de factores de paso corregidos ------------------------------------------------------
    if matches.is_present("gen_archivo_factores") {
        let path = Path::new(matches.value_of_os("gen_archivo_factores").unwrap());
        let fpstring = format!("{}", fpdata);
        if verbosity > 2 {
            println!("Factores de paso:\n{}", fpstring);
        }
        writefile(&path, fpstring.as_bytes());
        if verbosity > 0 {
            println!("Guardado archivo de factores de paso: {}", path.display());
        }
    }

    // Cálculo del balance -------------------------------------------------------------------------
    let balance: Option<Balance> = if components_is_some {
        Some(
            energy_performance(&components, &fpdata, kexp, arearef).unwrap_or_else(|error| {
                eprintln!("ERROR: No se ha podido calcular el balance energético");
                if verbosity > 2 {
                    println!("{}, {}", error.cause(), error.backtrace())
                };
                process::exit(exitcode::DATAERR);
            }),
        )
    } else if matches.is_present("gen_archivos_factores") {
        println!(
            "No se calcula el balance pero se ha generado el archivo de factores de paso {}",
            matches.value_of("gen_archivo_factores").unwrap()
        );
        None
    } else {
        println!("No se han definido datos suficientes para calcular el balance energético. Necesita definir al menos los componentes energéticos y los factores de paso");
        None
    };

    // Salida de resultados ------------------------------------------------------------------------
    if let Some(balance) = balance {
        // Guardar balance en formato json
        if matches.is_present("archivo_salida_json") {
            let path = Path::new(matches.value_of_os("archivo_salida_json").unwrap());
            if verbosity > 0 { println!("Resultados en formato JSON: \"{:?}\"", path.display()); }
            let json = serde_json::to_string_pretty(&balance)
                .unwrap_or_else(|error| {
                    eprintln!("ERROR: No se ha podido convertir el balance al formato JSON");
                    if verbosity > 2 {
                        println!("{:?}, {:?}", error.cause(), error.backtrace())
                    };
                    process::exit(exitcode::DATAERR);
                });
            writefile(&path, json.as_bytes());
        }
        // Guardar balance en formato XML
        if matches.is_present("archivo_salida_xml") {
            let path = Path::new(matches.value_of_os("archivo_salida_xml").unwrap());
            if verbosity > 0 { println!("Resultados en formato XML: \"{:?}\"", path.display()); }
            let xml = cte::balance_to_XML(&balance);
            writefile(&path, xml.as_bytes());
        }
        // Mostrar siempre en formato plain
        if matches.is_present("acsnrb") {
            println!("** Balance energético (servicio de ACS, perímetro próximo)");
        } else {
            println!("** Balance energético");
        }
        println!("{}", cte::balance_to_plain(&balance));
    };
}
