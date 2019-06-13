/*! #Valores reglamentarios para el DB-HE

Orientados al cumplimiento del DB-HE del (Código Técnico de la Edificación CTE).

Factores de paso basados en el consumo de energía primaria
Factores de paso constantes a lo largo de los intervalos de cálculo
*/

use crate::rennren::RenNren;

/// Valor por defecto del área de referencia.
pub const AREAREF_DEFAULT: f32 = 1.0;
/// Valor predefinido del factor de exportación. Valor reglamentario.
pub const KEXP_DEFAULT: f32 = 0.0;

/// Valores por defecto para factores de paso de redes de distrito 1.
pub const CTE_RED_DEFAULTS_RED1: RenNren = RenNren {
    ren: 0.0,
    nren: 1.3,
}; // RED1, RED, SUMINISTRO, A, ren, nren

/// Valores por defecto para factores de paso de redes de distrito 2.
pub const CTE_RED_DEFAULTS_RED2: RenNren = RenNren {
    ren: 0.0,
    nren: 1.3,
}; // RED2, RED, SUMINISTRO, A, ren, nren

/// Valores por defecto para exportación a la red (paso A) de electricidad cogenerada.
pub const CTE_COGEN_DEFAULTS_TO_GRID: RenNren = RenNren {
    ren: 0.0,
    nren: 2.5,
}; // ELECTRICIDAD, COGENERACION, A_RED, A, ren, nren

/// Valores por defecto para exportación a usos no EPB (paso A) de electricidad cogenerada.
pub const CTE_COGEN_DEFAULTS_TO_NEPB: RenNren = RenNren {
    ren: 0.0,
    nren: 2.5,
}; // ELECTRICIDAD, COGENERACION, A_NEPB, A, ren, nren

// Localizaciones válidas para CTE
// const CTE_LOCS: [&str; 4] = ["PENINSULA", "BALEARES", "CANARIAS", "CEUTAMELILLA"];

// Valores bien conocidos de metadatos:
// CTE_AREAREF -> num
// CTE_KEXP -> num
// CTE_LOCALIZACION -> str
// CTE_COGEN -> num, num
// CTE_RED1 -> num, num
// CTE_RED2 -> num, num

/// Factores de paso reglamentarios (RITE 20/07/2014) para Península.
pub const CTE_FP_PENINSULA: &str = "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: PENINSULA
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
FUELOIL, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red (Fueloil = Gasóleo)
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.414, 1.954 # Recursos usados para suministrar electricidad (PENINSULA) desde la red
";

/// Factores de paso reglamentarios (RITE 20/07/2014) para Baleares.
pub const CTE_FP_BALEARES: &str = "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: BALEARES
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
FUELOIL, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red (Fueloil = Gasóleo)
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.082, 2.968 # Recursos usados para suministrar electricidad (BALEARES) desde la red
";

/// Factores de paso reglamentarios (RITE 20/07/2014) para Canarias.
pub const CTE_FP_CANARIAS: &str = "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: CANARIAS
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
FUELOIL, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red (Fueloil = Gasóleo)
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.070, 2.924 # Recursos usados para suministrar electricidad (CANARIAS) desde la red
";

/// Factores de paso reglamentarios (RITE 20/07/2014) para Ceuta y Melilla.
pub const CTE_FP_CEUTAMELILLA: &str = "
#META CTE_FUENTE: CTE2013
#META CTE_LOCALIZACION: CEUTAMELILLA
#META CTE_FUENTE_COMENTARIO: Factores de paso del documento reconocido del RITE de 20/07/2014
MEDIOAMBIENTE, RED, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)
MEDIOAMBIENTE, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)
BIOCARBURANTE, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red (Biocarburante = biomasa densificada (pellets))
BIOMASA, RED, SUMINISTRO, A, 1.003, 0.034 # Recursos usados para suministrar el vector desde la red
BIOMASADENSIFICADA, RED, SUMINISTRO, A, 1.028, 0.085 # Recursos usados para suministrar el vector desde la red
CARBON, RED, SUMINISTRO, A, 0.002, 1.082 # Recursos usados para suministrar el vector desde la red
FUELOIL, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red (Fueloil = Gasóleo)
GASNATURAL, RED, SUMINISTRO, A, 0.005, 1.190 # Recursos usados para suministrar el vector desde la red
GASOLEO, RED, SUMINISTRO, A, 0.003, 1.179 # Recursos usados para suministrar el vector desde la red
GLP, RED, SUMINISTRO, A, 0.003, 1.201 # Recursos usados para suministrar el vector desde la red
ELECTRICIDAD, INSITU, SUMINISTRO, A, 1.000, 0.000 # Recursos usados para producir electricidad in situ
ELECTRICIDAD, COGENERACION, SUMINISTRO, A, 0.000, 0.000 # Recursos usados para suministrar la energía (0 porque se contabiliza el vector que alimenta el cogenerador)
ELECTRICIDAD, RED, SUMINISTRO, A, 0.072, 2.718 # Recursos usados para suministrar electricidad (CEUTA Y MELILLA) desde la red
";
