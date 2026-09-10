CARGO ?= cargo +1.98.1
ROOT ?= .
PRODUCER ?=
PRODUCER_ARGS ?=
QUIRE ?= quire
MODULE ?=

RUN = CARGO_BUILD_JOBS=2 $(CARGO) run --locked --quiet --bin quire-corpus -- --root "$(ROOT)"

.PHONY: help
help:
	@echo "  make bounds      - inventory, criterion coverage, and the GAP gate"
	@echo "  make digest      - corpus revision and per-case digests"
	@echo "  make score PRODUCER=<path> PRODUCER_ARGS='<repeated --producer-arg flags>'"
	@echo "  make refresh-criteria PRODUCER_REPO=<repo> - re-pin producer criteria"
	@echo "  make test        - Rust qualification suite"
	@echo "  make coverage    - Test Matrix rows vs the Rust suite"
	@echo "  make audit       - locked dependency license and advisory gates"
	@echo "  make verify      - local bounds, digest, tests, and optional score"
	@echo "  make qualify     - all local qualification gates"

.PHONY: bounds
bounds:
	$(RUN) bounds

.PHONY: digest
digest:
	$(RUN) digest

.PHONY: score
score:
	@test -n "$(PRODUCER)" || { echo "set PRODUCER=<executable> and repeat --producer-arg in PRODUCER_ARGS"; exit 2; }
	$(RUN) score --producer "$(PRODUCER)" $(PRODUCER_ARGS)

.PHONY: refresh-criteria
refresh-criteria:
	@test -n "$(PRODUCER_REPO)" || { echo "set PRODUCER_REPO=<path to producer repository>"; exit 2; }
	$(RUN) refresh-criteria --producer-repo "$(PRODUCER_REPO)"

.PHONY: test
test:
	CARGO_BUILD_JOBS=2 $(CARGO) test --locked -- --test-threads=2

.PHONY: coverage
coverage:
	@test -n "$(MODULE)" || { echo "set MODULE=<exact Quire module directory>"; exit 2; }
	$(RUN) check-coverage --quire "$(QUIRE)" --module "$(MODULE)"

.PHONY: audit
audit:
	cargo deny check
	cargo audit

.PHONY: verify
verify: bounds digest test
	@if [ -n "$(PRODUCER)" ]; then $(RUN) score --producer "$(PRODUCER)" $(PRODUCER_ARGS); else echo "no PRODUCER set — corpus state verified without a producer score"; fi

.PHONY: qualify
qualify: verify coverage audit
