# Cambios

Los principales cambios del proyecto se reflejan en este archivo.

El formato se basa en el descrito en [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), y refleja los cambios clasificados en:

- `Novedades`: para nueva funcionalidad.
- `Cambios`: para modificaciones de funcionalidad existente.
- `Obsoleto`: para funcionalidad anteriormente estable que será eliminada en versiones futuras.
- `Eliminaciones`: para funcionalidades obsoletas que han sido eliminadas en esta versión.
- `Correcciones`: para correcciones de errores.
- `Seguridad`: para invitar a la actualización en el caso de vulnerabilidades.

Este proyecto sigue, además, el [Versionado semántico](https://semver.org/spec/v2.0.0.html).

## []

### Novedades

- Posibilidad de excluir en el cálculo de la demanda renovable de ACS de
  los componentes marcados con `CTEEPBD_EXCLUYE_SCOP_ACS` en el comentario.
  Esto permite excluir los consumos de energía ambiente en casos de Bombas de calor con SCOP < 2.5

## [0.22.0] - 2020-09-17

### Correcciones

- Corrección del fragmento XML (\<CO2> --> \<co2>)

## [0.21.0] - 2020-06-30

### Cambios

- El cálculo del porcentaje renovable de la demanda de ACS en el perímetro próximo ahora admite más casos, con las siguientes restricciones:
  - Si se produce ACS con biomasa, solo se puede combinar con otra producción in situ
  - No admite producción de ACS con electricidad cogenerada

### Correcciones

- Se corrige el cálculo del porcentaje renovable de la demanda de ACS en el perímetro próximo, que en algunos casos se reportaba superior al 100%.

## [0.20.0] - 2020-06-13

### Cambios

- El cálculo de la parte renovable de la demanda de ACS en perímetro próximo es ahora una
  operación infalible (siempre devuelve un resultado). En el caso de que el indicador no se pueda calcular, se añade en `balance.misc` una clave `error_acs` donde se dan los detalles, evitando la salida abrupta del programa.

## [0.19.0] - 2020-06-12

### Novedades

- Posibilidad de excluir en el cálculo de la demanda renovable de ACS de
  los componentes marcados con `CTEEPBD_EXCLUYE_AUX_ACS` en el comentario.
  Esto permite excluir los consumos auxiliares eléctricos, que no
  contribuyen a la demanda.
- Posiblidad de indicar la demanda anual de ACS en los metadatos con `CTE_ACS_DEMANDA_ANUAL`

## [0.18.0] - 2020-06-03

### Novedades

- Documentación de los códigos de salida de la aplicación (OK y errores)

### Correcciones

- Detalla error al recibir demanda anual de ACS nula para el cálculo de la parte renovable de la demanda (emite error)

## [0.17.0] - 2020-05-13

### Correcciones

- Corrección del reparto de la producción eléctrica por servicios en presencia de consumos No EPB

## [0.16.0] - 2020-05-06

### Novedades

- Cálculo del indicador de cobertura renovable de la demanda de ACS (HE4).
  El indicador se obtiene cuando se indica la opción --demanda_anual_acs DEMANDA_TOTAL_kWh y se cumplen las restricciones para su cálculo:
  - un solo vector no in situ consumido para producir ACS
  - no se permite el uso de electricidad cogenerada
- Posibilidad de introducir factores de paso de exportación de electricidad cogenerada a usos no EPB

### Obsoleto

- Se oculta en la ayuda la opción de cálculo del balance para el perímetro próximo y para el servicio de ACS
  Esta opción sigue, de todos modos, disponible.

### Correcciones

- Al calcular la asignación por servicios se reparte también la electricidad cogenerada, además de la producida in situ

## Versión 0.15.0 (2020-03-04)

### Correcciones

- Corrección de referencias a fracción renovable de ACS, puesto que el DB-HE final se establece la exigencia en relación a la demanda y no a la fracción renovable del consumo de energía primaria.

## [0.14.0] - 2019-10-07

### Cambios

- Reducción del tamaño de los ejecutables
- Mejoras en la documentación

## [0.13.2] - 2019-07-31

### Novedades

- Cálculo simultáneo de energía primaria y emisiones (se elimina modo de cálculo), incluyendo valores desagregados
- Infraestructura para generación de bindings Wasm.

### Cambios

- Acotar el uso de metadatos en archivos de componentes (CTE_LOCALIZACION, CTE_KEXP, CTE_AREAREF, CTE_COGEN, CTE_COGENNEPB, CTE_RED1, CTE_RED2, CTE_FUENTE) y factores de paso (CTE_FUENTE, CTE_LOCALIZACION)

## [0.12.0] - 2019-07-22

### Cambios

- Mejoras en la notificación de errores en datos de entrada
- Ejemplo actualizado

## [0.11.0] - 2019-07-03

### Novedades

- Cálculo de emisiones con --modo CO2
- Cálculo de valores (energía final e indicadores) desagregados por servicios
- Incorporación del servicio BAC para automatización y control del edificio

### Cambios

- Mejora y actualización del manual. Aclaración sobre el uso para comprobación reglamentaria
- Comprobación de la producción de electricidad con uso NDEF (no se permiten usos específicos)
- Mejoras en la estructura de la salida JSON (mejora consistencia y añade nuevos campos)
- Cambio de la salida XML para el CTE DB-HE: uso de tot y nren en lugar de ren y nren
- Optimización del tamaño de los ejecutables

### Eliminaciones

- Eliminación del vector FUELOIL (era, en la práctica, igual a GASOLEO)
- Renombrado de tipos y subtipos: input -> SUMINISTRO, to_nEPB -> A_NEPB, to_grid -> A_RED
