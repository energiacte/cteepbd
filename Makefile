SRCDIR:=src
BUILDDIR:=target/debug
SCRIPT=cteepbd
TESTFP:=test_data/factores_paso_PENINSULA_20140203.csv
TESTCARRIERS:=test_data/cte_test_carriers.csv

test:
	#cargo test -- nocapture
	cargo test

run:
	cargo run

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