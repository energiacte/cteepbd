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

## [0.24.0] - Sin publicar

### Novedades

- Implementación del factor de coincidencia de cargas con criterio estadístico (se mantiene el factor constante salvo activación con opción --load-matching)
- Implementación de prioridades en el reparto de la energía producida a consumos EPB, con EL_INSITU >
- Se eliminan las categorías de servicio HU (humidificación) y DHU (deshumidificación), debiendo integrarse sus consumos en los de calefacción y/o refrigeración.
- Se elimina la categoría de servicio BAC (automatización y control), debiendo integrarse en los consumos auxiliares o corrientes de los servicios afectados.
- Cambios en el componente energético para energía usada (CONSUMO)
  - Añadido un identificador (un número entero) del sistema asociado al componente (id)
  - Eliminación de los subtipos EPB, NEPB
  - Los usos no EPB se identifican usando un nuevo servicio llamado NEPB. El resto, salvo el COGEN, son siempre servicios EPB.
- Cambios en el componente energético para energía producida (PRODUCCION)
  - Añadido un identificador (un número entero) del sistema asociado al componente (id)
  - Se elimina el vector energético
  - Se definen las siguientes fuentes: TERMOSOLAR, EAMBIENTE, EL_INSITU, EL_COGEN
- Nuevo componente energético para energía saliente (SALIDA)
  - Indica la energía absorbida (p.e. refrigeración) o entregada (p.e. calor) en cada paso de cálculo), para cada servicio y para cada sistema (id=i)
  - Incluye un identificador de sistema asociado (id), servicio, valores para los pasos de cálculo y un comentario.
  - Se utiliza para determinar la demanda generada de ACS generada en equipos de BIOMASA y BIOMASADENSIFICADA, eliminándose los metadatos
    `CTE_DEMANDA_ACS_PCT_BIOMASA` y `CTE_DEMANDA_ACS_PCT_BIOMASADENSIFICADA`.
- Nuevo componente energético para consumo auxiliar (AUX)
  - Indica la energía eléctrica consumida para usos auxiliares de los servicios EPB en cada paso de cálculo, para cada sistema (id=i)
  - Incluye un identificador de sistema asociado (id), valores para los pasos de cálculo y un comentario.
  - El reparto de este consumo entre servicios se realiza de forma automática y proporcionalmente a la energía entregada o absorbida por cada servicio EPB
  - Nota: este componente sustituye la anotación de consumos en el comentario con CTEEPBD_EXCLUYE_AUX_ACS, que deja de estar soportada
  - Cuando se definan consumos auxiliares el sistema debe, bien atender un solo servicio, o bien definir la energía saliente para cada uno de ellos.
- Nuevo elemento de datos de demanda del edificio (DEMANDA)
  - Permite definir la demanda del edificio para CAL, REF, ACS
    - Incluye: DEMANDA, servicio, datos de demanda para cada paso de cálculo (kWh)
    - Se elimina la introducción de la demanda anual de ACS con el metadato `CTE_ACS_DEMANDA_ANUAL`
- Nuevo vector energético TERMOSOLAR
  - Identifica la energía solar térmica procedente de captadores
  - La compensación automática de consumos de TERMOSOLAR se realiza sistema a sistema y servicio a servicio, sin traslado de energía entre ellos.
- Nuevo vector energético EAMBIENTE
  - Identifica la energía ambiente capturada por las bombas de calor
  - La compensación automática de consumos de EAMBIENTE se realiza sistema a sistema y servicio a servicio, sin traslado de energía entre ellos.
- Eliminación del vector energético MEDIOAMBIENTE (que se desdobla en EAMBIENTE y TERMOSOLAR)
- Eliminación del servicio NDEF
- Nuevo servicio NEPB
  - Para consumos destinados a usos no EPB
- Nuevo servicio COGEN
  - Para consumos destinados a la producción eléctrica por cogeneración
  - Estos consumos no pertenece ni a usos EPB ni a usos no EPB
  - Permite el cálculo de los factores de paso de la electricidad cogenerada
  - Se eliminan los factores de paso de usuario para la cogeneración (`CTE_COGEN`)
  - Es necesario indicar el consumo de vectores para la cogeneración eléctrica con componentes CONSUMO,COGEN,valores...
- Nueva salida XML
- Salida JSON:
  - Nuevos resultados disponibles
  - Nuevos resultados en balance y balance_m2:
    - needs: demanda energética, por servicio
    - used_EPB: energía final consumida en servicios EPB
    - used_nEPB: energía final consumida en servicios no EPB
    - prod: energía final producida
    - prod_by_src: energía final producida, por origen
    - prod_by_cr: energía final producida, por vector
    - del: energía final suministrada
    - exp: energía final exportada
    - exp_grid: energía final exportada a la red
    - exp_nEPB: energía final exportada a usos no EPB
  - Cambio de nombre de sufijos "\_bygen" a "\_by_src" y "\_byuse" a "\_by_srv" en la salida JSON
- Ejecutable cteepbd:
  - Eliminada la opción `--acsnrb` para el cálculo exclusivo de ACS en perímetro nearby (ya se calcula incondicionalmente)
  - Eliminada la opción `--demanda_anual_acs`, debiendo introducirse los datos mediante un componente `DEMANDA,ACS,...`
  - Añadida la opción `--load_matching` para realizar el cálculo de coincidencia de cargas (en lugar de usar f_match = 1)

### Incompatibilidades

- Cambios en el formato de salida en XML:
  - se incluye siempre la etiqueta `<Id>` de identificador de sistemas.+
  - los componentes de energía consumida se definen con una etiqueta `<Consumo>` y se elimina la etiqueta tipo `<Tipo>`
  - los componentes de energía generada se definen con una etiqueta `<Produccion>` y se elimina la etiqueta tipo `<Tipo>`
  - los componentes de demanda de zona se definen con una etiqueta `<Zona><Demanda>...</Demanda></Zona>` y se elimina la etiqueta tipo `<Tipo>`
  - los componentes de demanda sobre los equipos se definen con una etiqueta `<Sistema><Demanda>...</Demanda></Sistema>` y se elimina la etiqueta tipo `<Tipo>`
  - TODO: Revisar conversión a XML

## [0.23.0] - 2020-10-23

### Novedades

- Posibilidad de excluir en el cálculo de la demanda renovable de ACS de
  los componentes marcados con `CTEEPBD_EXCLUYE_SCOP_ACS` en el comentario.
  Esto permite excluir los consumos de energía ambiente en casos de Bombas de calor con SCOP < 2.5
- Posibilidad de calcular la fracción renovable de la demanda de ACS con cualquier combinación con equipos de biomasa.
  Si se usan distintos tipos de biomasa sólida o se combinan con vectores distintos a la energía ambiente o de redes de distrito, es necesario indicar en los metadatos
  el porcentaje de la demanda de ACS que se cubre con `BIOMASA` (clave `CTE_DEMANDA_ACS_PCT_BIOMASA`) y/o con `BIOMASADENSIFICADA` (clave `CTE_DEMANDA_ACS_PCT_BIOMASA`).
  Así, ahora únicamente no se podría calcular la fracción renovable de la demanda de ACS de aquellos casos con consumo de electricidad cogenerada.

### Correcciones

- Corrección del cálculo de la fracción de ACS con vectores de red de distrito `RED1` y `RED2` y cálculo más preciso para los vectores `BIOMASA` y `BIOMASADENSIFICADA`.

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
