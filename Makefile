SRCDIR:=src
BUILDDIR:=target/debug
SCRIPT=cteepbd
TESTDIR=test_data
TESTFP:=test_data/factores_paso_PENINSULA_20140203.csv
TESTCARRIERS:=test_data/cte_test_carriers.csv
PDFLATEX := $(shell which pdflatex 2> /dev/null)
# Rutas al archivo zip, relativas al directorio de distribución "dist".
OUTBUNDLE="../cteepbd-$(shell date +"%Y%m%d").zip"
OUTBUNDLEBAK = "../cteepbd-$(shell date +"%Y%m%d").zip.bak"

test:
	$(info [INFO]: Ejecución de tests)
	#cargo test -- nocapture
	cargo test

run:
	$(info [INFO]: Ejecutando versión de depuración)
	cargo run

build:
	$(info [INFO]: Compilando ejecutable (versión de depuración))
	cargo build

linux:
	$(info [INFO]: Versión de producción para linux)
	cargo build --release

win32:
	$(info [INFO]: Versión de producción para i686-pc-windows-gnu)
	cargo build --release --target=i686-pc-windows-gnu

release: linux win32
	$(info [INFO]: Compilando versión de producción)
	mkdir -p dist
	cp target/i686-pc-windows-gnu/release/cteepbd.exe dist/
	cp target/release/cteepbd dist/
	strip dist/cteepbd.exe
	strip dist/cteepbd

clippy:
	$(info [INFO]: Comprobaciones con clippy)
	cargo +nightly clippy

bloat:
	$(info [INFO]: Calculando consumo de espacio en archivo ejecutable)
	cargo bloat --release -n 10
	cargo bloat --release --crates -n 10

cteepbd: build
	$(info [INFO]: Ejemplos de prueba mínimos)
	${BUILDDIR}/${SCRIPT} --help
	${BUILDDIR}/${SCRIPT} -vv -c ${TESTCARRIERS} -f ${TESTFP} -a 200 --json balance.json --xml balance.xml > balance.txt
	${BUILDDIR}/${SCRIPT} -vv -c ${TESTCARRIERS} -l PENINSULA --cogen 0 2.5 --red1 0 1.3 --red2 0 1.3
	${BUILDDIR}/${SCRIPT} -vv -c ${TESTCARRIERS} -l PENINSULA
	${BUILDDIR}/${SCRIPT} -c ${TESTCARRIERS} -l PENINSULA --acs_nearby

FPTEST=test_data/factores_paso_test.csv
docexamples: linux
	$(info [INFO]: Generando ejemplos y archivos para el manual)
	#$(PYTHON) $(TESTDIR)/createfiles.py
	mkdir -p $(TESTDIR)/output
	target/release/cteepbd -c $(TESTDIR)/ejemploJ1_base.csv -l PENINSULA > $(TESTDIR)/output/ejemploJ1_base_pen.out
	target/release/cteepbd -c $(TESTDIR)/ejemploJ1_base.csv -f $(FPTEST) > $(TESTDIR)/output/ejemploJ1_base.out
	target/release/cteepbd -c $(TESTDIR)/ejemploJ2_basePV.csv -f $(FPTEST) > $(TESTDIR)/output/ejemploJ2_basePV.out
	target/release/cteepbd -c $(TESTDIR)/ejemploJ3_basePVexcess.csv -f $(FPTEST) > $(TESTDIR)/output/ejemploJ3_basePVexcess.out
	target/release/cteepbd -c $(TESTDIR)/ejemploJ5_gasPV.csv -f $(FPTEST) > $(TESTDIR)/output/ejemploJ5_gasPV.out
	target/release/cteepbd -c $(TESTDIR)/ejemploJ6_HPPV.csv -f $(FPTEST) > $(TESTDIR)/output/ejemploJ6_HPPV.out
	target/release/cteepbd -c $(TESTDIR)/ejemploJ7_cogenfuelgasboiler.csv -f $(FPTEST) > $(TESTDIR)/output/ejemploJ7_cogenfuelgasboiler.out
	target/release/cteepbd -c $(TESTDIR)/ejemploJ8_cogenbiogasboiler.csv -f $(FPTEST) > $(TESTDIR)/output/ejemploJ8_cogenbiogasboiler.out
	target/release/cteepbd -c $(TESTDIR)/ejemploJ9_electr.csv -f $(FPTEST) > $(TESTDIR)/output/ejemploJ9_electr.out
	target/release/cteepbd -h|fold -s -w105 > $(TESTDIR)/output/salida_ayuda.txt
	target/release/cteepbd -c $(TESTCARRIERS) -f $(FPTEST) > $(TESTDIR)/output/cte_test_carriers.out
	target/release/cteepbd -N -c $(TESTCARRIERS) -l PENINSULA > $(TESTDIR)/output/cte_test_carriers_ACS.out
	target/release/cteepbd -c $(TESTCARRIERS) -l PENINSULA --json "$(TESTDIR)/output/balance.json" --xml "$(TESTDIR)/output/balance.xml" > "$(TESTDIR)/output/balance.plain"

docs: docexamples docs/Manual_cteepbd.tex
	$(info [INFO]: Generando manual)
ifndef PDFLATEX
	$(error "Es necesario tener instalado pdflatex para generar la documentación")
endif
	cd docs && pdflatex --output-directory=build Manual_cteepbd.tex && pdflatex --output-directory=build Manual_cteepbd.tex
	cp docs/build/Manual_cteepbd.pdf ./dist

examples:
	$(info [INFO]: Copiando archivos de ejemplo)
	mkdir -p dist/test_data
	cp test_data/*.csv dist/test_data

bundle: release docs examples
	$(info [INFO]: Generando archivo .zip de distribución)
	cp LICENSE dist/LICENSE
	cp README.md dist/README.md
	-cd dist && [ -e $(OUTBUNDLE) ] && mv $(OUTBUNDLE) $(OUTBUNDLEBAK)
	cd dist && zip -r $(OUTBUNDLE) ./*

genjson:
	cargo run -- -c $(TESTCARRIERS) -l PENINSULA --json "prueba.json"