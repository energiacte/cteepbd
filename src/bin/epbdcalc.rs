extern crate epbdrs;
//#[macro_use]
extern crate failure;

use std::fs::File;
use std::io::prelude::*;

use epbdrs::epbd::*;

use failure::Error;
use failure::ResultExt;

fn readfile(path: &str) -> Result<String, Error> {
    let mut f = File::open(path).context(format!("Archivo {} no encontrado", path))?;
    let mut contents = String::new();
    f.read_to_string(&mut contents).context("Error al leer el archivo")?;
    Ok(contents)
}

fn main() {
    println!("Cargo manifest dir: {}", env!("CARGO_MANIFEST_DIR"));
    let cstr = readfile("src/examples/ejemplo3PVBdC.csv").unwrap();
    let components = cstr.parse().unwrap();
    let fstr = readfile("src/examples/factores_paso_20140203.csv").unwrap();
    let wfactors = fstr.parse().unwrap();
    let k_exp = 1.0;
    let arearef = 1.0;
    let res = energy_performance(components, wfactors, k_exp, arearef);
    println!("{:#?}", res);
    ()
}
