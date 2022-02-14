# CteEPBD

Library implementation and CLI of the ISO EN 52000-1 "Energy performance of buildings" standard
to explore NZEB indicators.

Programa de cálculo de la eficiencia energética de los edificios para su aplicación al
CTE DB-HE (procedimiento EN ISO 52000-1) y formatos de datos

## Introducción

Este programa, `CteEPBD`, implementa la metodología de cálculo de la eficiencia energética
de los edificios descrita en la norma EN ISO 52000-1:2017 de
_Eficiencia energética de los edificios. Evaluación global. Parte 1: Marco general y procedimientos_
dentro del alcance de la _Directiva 2010/31/UE_ relativa a la eficiencia
de los edificios (EPDB) y del _Documento Básico de Ahorro de Energía_ (_DB-HE_) del
_Código Técnico de la Edificación_ (_CTE_).

El programa calcula la energía suministrada al edificio (desde redes de abastecimiento o
producida _in situ_) y la energía exportada (a la red y a usos no EPB) para obtener diversos
indicadores de la eficiencia energética del edificio, expresada como energía ponderada
(p.e. consumo de energía primaria no renovable, consumo de energía primaria total,
fracción renovable del consumo de energía primaria o emisiones de CO2). Para ello, toma
en consideración los factores de paso de los distintos vectores energéticos y el
factor de exportación (_k_exp_).

En algunos casos también permite calcular el porcentaje de la demanda de ACS de origen
renovable (no calcula este indicador cuando se usa biomasa con otros vectores
no producidos in situ o cuando se produce ACS con electricidad de cogeneración).

## Uso

El programa es autodocumentado y puede obtenerse ayuda usando la opción `-h`.

Una llamada típica al programa:

`$ cteepbd -c test_data/cte_test_carriers.csv -l PENINSULA`

Produce los siguientes resultados por pantalla:

```language-plain

    ** Datos de entrada

    Componentes energéticos: "test_data/cte_test_carriers.csv"
    Factores de paso (usuario): PENINSULA
    Área de referencia (metadatos) [m2]: 200.00
    Factor de exportación (metadatos) [-]: 0.0

    ** Eficiencia energética

    Area_ref = 200.00 [m2]
    k_exp = 0.00
    C_ep [kWh/m2.an]: ren = 24.6, nren = 18.9, tot = 43.5
    E_CO2 [kg_CO2e/m2.an]: 3.20
    RER = 0.57

    ** Energía final (todos los vectores) [kWh/m2.an]:

    Energía consumida: 30.25

    Consumida en usos EPB: 30.25

    * por servicio:
    - ACS: 11.22
    - CAL: 12.94
    - REF: 0.28
    - VEN: 5.81

    * por vector:
    - EAMBIENTE: 12.82
    - ELECTRICIDAD: 13.20
    - TERMOSOLAR: 4.23

    Consumida en usos no EPB: 0.00

    Generada: 20.58

    * por origen:
    - INSITU: 20.58

    * por vector:
    - EAMBIENTE: 12.82
    - ELECTRICIDAD: 3.53
    - TERMOSOLAR: 4.23

    Suministrada 30.25:

    - de red: 9.68
    - in situ: 20.58

    Exportada: 0.00

    - a la red: 0.00
    - a usos no EPB: 0.00

    ** Energía primaria (ren, nren) [kWh/m2.an] y emisiones [kg_CO2e/m2.an]:

    Recursos utilizados (paso A): ren 24.58, nren 18.91, tot: 43.49, co2: 3.20

    * por servicio:
    - ACS: ren 10.02, nren 4.01, tot: 14.03, co2: 0.68
    - CAL: ren 11.09, nren 6.18, tot: 17.26, co2: 1.05
    - REF: ren 0.16, nren 0.40, tot: 0.56, co2: 0.07
    - VEN: ren 3.32, nren 8.33, tot: 11.64, co2: 1.41

    Incluyendo el efecto de la energía exportada (paso B): ren 24.58, nren 18.91, tot: 43.49, co2: 3.20

    * por servicio:
    - ACS: ren 10.02, nren 4.01, tot: 14.03, co2: 0.68
    - CAL: ren 11.09, nren 6.18, tot: 17.26, co2: 1.05
    - REF: ren 0.16, nren 0.40, tot: 0.56, co2: 0.07
    - VEN: ren 3.32, nren 8.33, tot: 11.64, co2: 1.41

    ** Indicadores adicionales

    Demanda total de ACS: - [kWh]
    Porcentaje renovable de la demanda de ACS (perímetro próximo): - [%]
```

Donde se puede apreciar el resultado del cálculo del consumo de energía primaria renovable (C_ep_ren),
no renovable (C_ep_nren), total (C_ep_tot), la fracción renovable de energía primaria (RER)
y las emisiones de CO2 (E_CO2).

## Hipótesis de cálculo

Se han adoptado las siguientes hipótesis de cálculo y simplificaciones:

- los factores de paso son constantes a lo largo de los pasos de cálculo
- no se definen prioridades para la generación energética (f_we_el_stepA promedio)
- se considera como suministrada toda la energía producida por fuentes distintas a la cogeneración
- factor de coincidencia de cargas (f_match_t) constante o variable
- el reparto de energía eléctrica producida entre servicios es proporcional al consumo eléctrico
  servicio respecto al consumo total de servicios EPB
- los consumos auxiliares de un sistema se asignan a los servicios proveídos por dicho sistema
  de forma proporcional a la energía entregada o absorbida los dichos servicios EPB que provee.
- para el cálculo del porcentaje renovable de la demanda de ACS se considera que el rendimiento
  térmico de las aportaciones renovables distintas a la biomasa es igual a 1.0.
