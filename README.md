# CteEPBD

Library implementation and CLI of the ISO EN 52000-1 "Energy performance of buildings" standard to explore NZEB indicators.

Programa de cálculo de la eficiencia energética de los edificios para su aplicación al CTE DB-HE (procedimiento EN ISO 52000-1) y formatos de datos

## Introducción

Este programa, `CteEPBD`, implementa la metodología de cálculo de la eficiencia energética de los edificios descrita en la norma EN ISO 52000-1:2017 de *Eficiencia energética de los edificios. Evaluación global. Parte 1: Marco general y procedimientos* dentro del alcance de la *Directiva 2010/31/UE* relativa a la eficiencia energética de los edificios (EPDB) y del *Documento Básico de Ahorro de Energía* (*DB-HE*) del *Código Técnico de la Edificación* (*CTE*).

El programa calcula la energía suministrada al edificio (desde redes de abastecimiento o producida *in situ*) y la energía exportada (a la red y a usos no EPB) para obtener diversos indicadores de la eficiencia energética del edificio, expresada como energía ponderada (p.e. consumo de energía primaria no renovable, consumo de energía primaria total, fracción renovable del consumo de energía primaria o emisiones de CO2). Para ello, toma en consideración los factores de paso de los distintos vectores energéticos y el factor de exportación (*k_exp*).

En algunos casos también permite calcular el porcentaje de la demanda de ACS de origen renovable (no calcula este indicador cuando se usa biomasa con otros vectores
no producidos in situ o cuando se produce ACS con electricidad de cogeneración).

## Uso

El programa es autodocumentado y puede obtenerse ayuda usando la opción `-h`.

Una llamada típica al programa:

```$ cteepbd -c test_data/cte_test_carriers.csv -l PENINSULA```

Produce los siguientes resultados por pantalla:

```language-plain

    ** Datos de entrada
    Componentes energéticos: "test_data/cte_test_carriers.csv"
    Factores de paso (usuario): PENINSULA
    Área de referencia (metadatos) [m2]: 200.00
    Factor de exportación (metadatos) [-]: 0.0
    ** Balance energético
    Area_ref = 200.00 [m2]
    k_exp = 0.00
    C_ep [kWh/m2.an]: ren = 24.6, nren = 18.9, tot = 43.5, RER = 0.57
    E_CO2 [kg_CO2e/m2.an]: 3.20

    ** Energía final (todos los vectores) [kWh/m2.an]:
    ACS: 11.22
    CAL: 12.94
    REF: 0.28
    VEN: 5.81

    ** Energía primaria (ren, nren) [kWh/m2.an] y emisiones [kg_CO2e/m2.an] por servicios:
    ACS: ren 10.02, nren 4.01, co2: 0.68
    CAL: ren 11.09, nren 6.18, co2: 1.05
    REF: ren 0.16, nren 0.40, co2: 0.07
    VEN: ren 3.32, nren 8.33, co2: 1.41

```

Donde se puede apreciar el resultado del cálculo del consumo de energía primaria renovable (C_ep_ren), no renovable (C_ep_nren), total (C_ep_tot), la fracción renovable de energía primaria (RER) y las emisiones de CO2 (E_CO2).

## Hipótesis de cálculo

Se han adoptado las siguientes hipótesis de cálculo y simplificaciones:

- los factores de paso son constantes a lo largo de los pasos de cálculo
- no se definen prioridades para la generación energética (f_we_el_stepA promedio)
- se considera como suministrada toda la energía producida por fuentes distintas a la cogeneración
- el factor de coincidencia de cargas (f_match_t) se fija igual a 1.0
- no se asignan los consumos y producciones de energía a sistemas concretos (no son identificables)
- el reparto de energía eléctrica producida entre servicios es proporcional al consumo eléctrico del servicio respecto al total
- para el cálculo del porcentaje renovable de la demanda de ACS se considera que el rendimiento térmico de las aportaciones
  renovables distintas a la biomasa es igual a 1.0.
