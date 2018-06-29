SRCDIR:=src
BUILDDIR:=target/debug
SCRIPT=cteepbd
TESTDIR=test_data
TESTFP:=test_data/factores_paso_PENINSULA_20140203.csv
TESTCARRIERS:=test_data/cte_test_carriers.csv

test:
	#cargo test -- nocapture
	cargo test

run:
	cargo run

linux:
	cargo build --release

win32:
	cargo build --release --target=i686-pc-windows-gnu

build: linux win32
	mkdir -p dist
	cp target/i686-pc-windows-gnu/release/cteepbd.exe dist/
	cp target/release/cteepbd dist/
	strip dist/cteepbd.exe
	strip dist/cteepbd

clippy:
	cargo +nightly clippy

updateclippy:
	cargo +nightly install --force clippy
	#cargo +nightly install clippy --force --git https://github.com/rust-lang-nursery/rust-clippy.git
bloat:
	cargo bloat --release -n 10
	cargo bloat --release --crates -n 10

cteepbd:
	${BUILDDIR}/${SCRIPT} --help
	${BUILDDIR}/${SCRIPT} -vv -c ${TESTCARRIERS} -f ${TESTFP} -a 200 --json balance.json --xml balance.xml > balance.txt
	${BUILDDIR}/${SCRIPT} -vv -c ${TESTCARRIERS} -l PENINSULA --cogen 0 2.5 --red1 0 1.3 --red2 0 1.3
	${BUILDDIR}/${SCRIPT} -vv -c ${TESTCARRIERS} -l PENINSULA
	${BUILDDIR}/${SCRIPT} -c ${TESTCARRIERS} -l PENINSULA --acs_nearby

FPTEST=test_data/factores_paso_test.csv
createtest: linux
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
	target/release/cteepbd -c $(TESTDIR)/cte_test_carriers.csv -f $(FPTEST) > $(TESTDIR)/output/cte_test_carriers.out
	target/release/cteepbd -N -c $(TESTDIR)/cte_test_carriers.csv -l PENINSULA > $(TESTDIR)/output/cte_test_carriers_ACS.out
	target/release/cteepbd -c "$(TESTDIR)/cte_test_carriers.csv" -l PENINSULA --json "$(TESTDIR)/output/balance.json" --xml "$(TESTDIR)/output/balance.xml" > "$(TESTDIR)/output/balance.plain"
