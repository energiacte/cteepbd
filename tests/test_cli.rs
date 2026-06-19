#[test]
fn ejemplo_j1_loc() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "test_data/ejemploJ1_base.csv", "-l", "PENINSULA"])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 41.4, nren = 195.4, tot = 236.8, RER = 0.17")
        .unwrap();
}

#[test]
fn ejemplo_j1() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ1_base.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 50.0, nren = 200.0, tot = 250.0, RER = 0.20")
        .unwrap();
}

#[test]
fn ejemplo_j2() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ2_basePV.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 75.0, nren = 100.0, tot = 175.0, RER = 0.43")
        .unwrap();
}

#[test]
fn ejemplo_j3() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ3_basePVexcess.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 100.0, nren = 0.0, tot = 100.0, RER = 1.00")
        .unwrap();
}

#[test]
fn ejemplo_j5() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ5_gasPV.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 20.0, nren = 209.0, tot = 229.0, RER = 0.09")
        .unwrap();
}

#[test]
fn ejemplo_j6() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ6_HPPV.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 180.5, nren = 38.0, tot = 218.5, RER = 0.83")
        .unwrap();
}

#[test]
fn ejemplo_j7() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ7_cogenfuelgasboiler.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = -27.4, nren = 283.8, tot = 256.4, RER = -0.11")
        .unwrap();
}

#[test]
fn ejemplo_j8() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ8_cogenbiogasboiler.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 146.4, nren = 125.8, tot = 272.2, RER = 0.54")
        .unwrap();
}

#[test]
fn ejemplo_j9() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/ejemploJ9_electr.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 1009.5, nren = 842.0, tot = 1851.5, RER = 0.55")
        .unwrap();
}

#[test]
fn ejemplo_testcarriers() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/cte_test_carriers.csv",
            "-f",
            "test_data/factores_paso_test.csv",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 25.4, nren = 19.4, tot = 44.8, RER = 0.57")
        .unwrap();
}

#[test]
fn ejemplo_testcarriers_loc() {
    assert_cli::Assert::main_binary()
        .with_args(&["-c", "test_data/cte_test_carriers.csv", "-l", "PENINSULA"])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 24.6, nren = 18.9, tot = 43.5, RER = 0.57")
        .unwrap();
}

#[test]
fn ejemplo_testcarriers_loc_nearby() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/cte_test_carriers.csv",
            "-l",
            "PENINSULA",
            "--acs_nearby",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 9.2, nren = 4.7, tot = 13.9, RER = 0.66")
        .unwrap();
}

#[test]
fn ejemplo_acs_demanda_ren_con_nepb() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/acs_demanda_ren_con_nepb.csv",
            "-l",
            "PENINSULA",
            "--demanda_anual_acs",
            "1823.8",
        ])
        .stdout()
        .contains("Porcentaje renovable de la demanda de ACS (perímetro próximo): 77.3 [%]")
        .unwrap();
}

#[test]
fn ejemplo_acs_demanda_ren_con_nepb_con_exclusion_aux() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/acs_demanda_ren_con_exclusion_auxiliares.csv",
            "-l",
            "PENINSULA",
            "--demanda_anual_acs",
            "4549.0",
        ])
        .stdout()
        .contains("Porcentaje renovable de la demanda de ACS (perímetro próximo): 96.7 [%]")
        .unwrap();
}

#[test]
fn ejemplo_acs_demanda_ren_con_nepb_con_exclusion_aux_meta() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/acs_demanda_ren_con_exclusion_auxiliares.csv",
            "-l",
            "PENINSULA",
        ])
        .stdout()
        .contains("Porcentaje renovable de la demanda de ACS (perímetro próximo): 96.7 [%]")
        .unwrap();
}

#[test]
fn ejemplo_pvbdc_fp_peninsula() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/extra/ejemplo3PVBdC.csv",
            "-f",
            "test_data/factores_paso_PENINSULA_20140203.csv",
            "-k",
            "1.0",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 178.9, nren = 37.1, tot = 216.0, RER = 0.83")
        .unwrap();
}

#[test]
fn ejemplo_pvbdc_fp_peninsula_pef0onsite() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/extra/ejemplo3PVBdC.csv",
            "-f",
            "test_data/factores_paso_PENINSULA_20140203_pef0onsite.csv",
            "-k",
            "1.0",
        ])
        .stdout()
        .contains("C_ep [kWh/m2.an]: ren = 7.9, nren = 37.1, tot = 45.0, RER = 0.17")
        .unwrap();
}

#[test]
fn ejemplo_salida_xml30() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/extra/ejemplo3PVBdC.csv",
            "-f",
            "test_data/factores_paso_PENINSULA_20140203_pef0onsite.csv",
            "-k",
            "1.0",
            "--xml",
            "/dev/null",
            "-vvvv", // Con verbosity >0 se imprime el XML
            "--xml_version",
            "3.0"
        ])
        .stdout()
        .contains("<DatosBalance>    
    <kexp>1.00</kexp>
    <AreaRef>1.00</AreaRef><!-- área de referencia [m2] -->
    <FactoresPaso>
        <Version>3.0</Version>
        <Metadatos>
            <Metadato><Clave>CTE_FUENTE</Clave><Valor>RITE2014</Valor></Metadato>
            <Metadato><Clave>CTE_FUENTE_COMENTARIO</Clave><Valor>Factores de paso del documento reconocido del RITE de 20/07/2014</Valor></Metadato>
            <Metadato><Clave>CTE_LOCALIZACION</Clave><Valor>PENINSULA</Valor></Metadato>
        </Metadatos>
        <Datos>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Origen>RED</Origen><Destino>SUMINISTRO</Destino><Paso>A</Paso>
                <ren>0.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)</Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Origen>INSITU</Origen><Destino>SUMINISTRO</Destino><Paso>A</Paso>
                <ren>0.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)</Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Origen>INSITU</Origen><Destino>A_RED</Destino><Paso>A</Paso>
                <ren>1.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para producir la energía exportada a la red</Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Origen>INSITU</Origen><Destino>A_RED</Destino><Paso>B</Paso>
                <ren>1.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos ahorrados a la red por la energía producida in situ y exportada a la red</Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Origen>RED</Origen><Destino>SUMINISTRO</Destino><Paso>A</Paso>
                <ren>0.414</ren><nren>1.954</nren><co2>0.331</co2>
                <Comentario>Recursos usados para suministrar electricidad (PENINSULA) desde la red</Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Origen>INSITU</Origen><Destino>SUMINISTRO</Destino><Paso>A</Paso>
                <ren>0.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para producir electricidad in situ</Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Origen>INSITU</Origen><Destino>A_RED</Destino><Paso>A</Paso>
                <ren>1.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para producir la energía exportada a la red</Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Origen>INSITU</Origen><Destino>A_RED</Destino><Paso>B</Paso>
                <ren>0.414</ren><nren>1.954</nren><co2>0.331</co2>
                <Comentario>Recursos ahorrados a la red por la energía producida in situ y exportada a la red</Comentario>
            </Dato>
        </Datos>
    </FactoresPaso>
    <Componentes>
        <Version>3.0</Version>
        <Metadatos>
            <Metadato><Clave>CTE_AREAREF</Clave><Valor>1.00</Valor></Metadato>
            <Metadato><Clave>CTE_KEXP</Clave><Valor>1.0</Valor></Metadato>
        </Metadatos>
        <Datos>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Tipo>CONSUMO</Tipo><Subtipo>EPB</Subtipo><Servicio>NDEF</Servicio>
                <Valores>9.67 7.74 4.84 4.35 2.42 2.90 3.87 3.39 2.42 3.87 5.80 7.74</Valores>
                <Comentario></Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Tipo>PRODUCCION</Tipo><Subtipo>INSITU</Subtipo><Servicio>NDEF</Servicio>
                <Valores>1.13 1.42 1.99 2.84 4.82 5.39 5.67 5.11 4.54 3.40 2.27 1.42</Valores>
                <Comentario></Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Tipo>CONSUMO</Tipo><Subtipo>EPB</Subtipo><Servicio>NDEF</Servicio>
                <Valores>21.48 17.18 10.74 9.66 5.37 6.44 8.59 7.52 5.37 8.59 12.89 17.18</Valores>
                <Comentario></Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Tipo>PRODUCCION</Tipo><Subtipo>INSITU</Subtipo><Servicio>NDEF</Servicio>
                <Valores>21.48 17.18 10.74 9.66 5.37 6.44 8.59 7.52 5.37 8.59 12.89 17.18</Valores>
                <Comentario></Comentario>
            </Dato>
        </Datos>
    </Componentes>
</DatosBalance>
<ResultadosBalance>
    <Epm2><!-- C_ep [kWh/m2.an] -->
        <tot>45.0</tot>
        <nren>37.1</nren>
    </Epm2>
</ResultadosBalance>")
        .unwrap();
}

#[test]
fn ejemplo_salida_xml21() {
    assert_cli::Assert::main_binary()
        .with_args(&[
            "-c",
            "test_data/extra/ejemplo3PVBdC.csv",
            "-f",
            "test_data/factores_paso_PENINSULA_20140203_pef0onsite.csv",
            "-k",
            "1.0",
            "--xml",
            "/dev/null",
            "-vvvv", // Con verbosity >0 se imprime el XML
            "--xml_version",
            "2.1"
        ])
        .stdout()
        .contains("<BalanceEPB>
    <FactoresDePaso>
        <Metadatos>
            <Metadato><Clave>CTE_FUENTE</Clave><Valor>RITE2014</Valor></Metadato>
            <Metadato><Clave>CTE_FUENTE_COMENTARIO</Clave><Valor>Factores de paso del documento reconocido del RITE de 20/07/2014</Valor></Metadato>
            <Metadato><Clave>CTE_LOCALIZACION</Clave><Valor>PENINSULA</Valor></Metadato>
        </Metadatos>
        <Datos>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Origen>RED</Origen><Destino>SUMINISTRO</Destino><Paso>A</Paso>
                <ren>0.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para suministrar energía térmica del medioambiente (red de suministro ficticia)</Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Origen>INSITU</Origen><Destino>SUMINISTRO</Destino><Paso>A</Paso>
                <ren>0.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para generar in situ energía térmica del medioambiente (vector renovable)</Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Origen>INSITU</Origen><Destino>A_RED</Destino><Paso>A</Paso>
                <ren>1.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para producir la energía exportada a la red</Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Origen>INSITU</Origen><Destino>A_RED</Destino><Paso>B</Paso>
                <ren>1.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos ahorrados a la red por la energía producida in situ y exportada a la red</Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Origen>RED</Origen><Destino>SUMINISTRO</Destino><Paso>A</Paso>
                <ren>0.414</ren><nren>1.954</nren><co2>0.331</co2>
                <Comentario>Recursos usados para suministrar electricidad (PENINSULA) desde la red</Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Origen>INSITU</Origen><Destino>SUMINISTRO</Destino><Paso>A</Paso>
                <ren>0.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para producir electricidad in situ</Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Origen>INSITU</Origen><Destino>A_RED</Destino><Paso>A</Paso>
                <ren>1.000</ren><nren>0.000</nren><co2>0.000</co2>
                <Comentario>Recursos usados para producir la energía exportada a la red</Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Origen>INSITU</Origen><Destino>A_RED</Destino><Paso>B</Paso>
                <ren>0.414</ren><nren>1.954</nren><co2>0.331</co2>
                <Comentario>Recursos ahorrados a la red por la energía producida in situ y exportada a la red</Comentario>
            </Dato>
        </Datos>
    </FactoresDePaso>
    <Componentes>
        <Metadatos>
            <Metadato><Clave>CTE_AREAREF</Clave><Valor>1.00</Valor></Metadato>
            <Metadato><Clave>CTE_KEXP</Clave><Valor>1.0</Valor></Metadato>
        </Metadatos>
        <Datos>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Tipo>CONSUMO</Tipo><Subtipo>EPB</Subtipo><Servicio>NDEF</Servicio>
                <Valores>9.67,7.74,4.84,4.35,2.42,2.90,3.87,3.39,2.42,3.87,5.80,7.74</Valores>
                <Comentario></Comentario>
            </Dato>
            <Dato>
                <Vector>ELECTRICIDAD</Vector><Tipo>PRODUCCION</Tipo><Subtipo>INSITU</Subtipo><Servicio>NDEF</Servicio>
                <Valores>1.13,1.42,1.99,2.84,4.82,5.39,5.67,5.11,4.54,3.40,2.27,1.42</Valores>
                <Comentario></Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Tipo>CONSUMO</Tipo><Subtipo>EPB</Subtipo><Servicio>NDEF</Servicio>
                <Valores>21.48,17.18,10.74,9.66,5.37,6.44,8.59,7.52,5.37,8.59,12.89,17.18</Valores>
                <Comentario></Comentario>
            </Dato>
            <Dato>
                <Vector>MEDIOAMBIENTE</Vector><Tipo>PRODUCCION</Tipo><Subtipo>INSITU</Subtipo><Servicio>NDEF</Servicio>
                <Valores>21.48,17.18,10.74,9.66,5.37,6.44,8.59,7.52,5.37,8.59,12.89,17.18</Valores>
                <Comentario></Comentario>
            </Dato>
        </Datos>
    </Componentes>
    <kexp>1.00</kexp>
    <AreaRef>1.00</AreaRef><!-- área de referencia [m2] -->
    <Epm2><!-- C_ep [kWh/m2.an] -->
        <tot>45.0</tot>
        <nren>37.1</nren>
    </Epm2>
</BalanceEPB>")
        .unwrap();
}