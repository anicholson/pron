NEXTEST_JSON := NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1
TREEFMT := | cargo run -p xtask --
JSONFMT := --message-format libtest-json --message-format-version 0.1 --no-tests pass

.PHONY: test test-lib test-integration test-flat test-watch test-mutate test-ci lint

test:
	$(NEXTEST_JSON) cargo nextest run $(JSONFMT) $(TREEFMT)

test-lib:
	$(NEXTEST_JSON) cargo nextest run --lib $(JSONFMT) $(TREEFMT)

test-integration:
	$(NEXTEST_JSON) cargo nextest run -E 'kind(test)' $(JSONFMT) $(TREEFMT)

test-flat:
	cargo nextest run

test-watch:
	watchexec -e rs -- make test-lib

test-mutate:
	cargo mutants -p pron -e src/main.rs -- --lib --no-tests pass

test-ci:
	cargo nextest run --profile ci

lint:
	cargo clippy -- -D warnings
