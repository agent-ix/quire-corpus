# The corpus is static files. These targets derive state from them and never
# write any back — a stored count is a number that can go stale.

PRODUCER ?=

.PHONY: help
help:
	@echo "  make bounds     - inventory vs the tree; fails on a GAP or an undeclared fixture"
	@echo "  make digest     - corpus revision and per-case digests"
	@echo "  make score      - score a producer:  make score PRODUCER='<cmd> --org {org} --repo {repo} {input}'"
	@echo "  make test       - the corpus's own gates, and the guards that keep them able to fail"
	@echo "  make coverage   - Test Matrix rows vs the suite (quire coverage)"
	@echo "  make verify     - bounds + digest + test, then score when PRODUCER is set"
	@echo "  make ci         - verify + coverage"

.PHONY: bounds
bounds:
	python3 bounds.py

.PHONY: digest
digest:
	python3 digest.py

.PHONY: score
score:
	@test -n "$(PRODUCER)" || { echo "set PRODUCER='<cmd> --org {org} --repo {repo} {input}'"; exit 2; }
	python3 score.py --producer '$(PRODUCER)'

.PHONY: test
test:
	python3 tests/run.py

# Every matrix row is backed by a tagged test, or its own declared verification
# method says why no symbol can exist. Needs `quire` on PATH and a module path
# declaring the traceability model.
.PHONY: coverage
coverage:
	bash scripts/check_coverage.sh

.PHONY: verify
verify: bounds digest test
	@if [ -n "$(PRODUCER)" ]; then python3 score.py --producer '$(PRODUCER)'; \
	else echo "no PRODUCER set — inventory and digests checked, nothing scored"; fi

.PHONY: ci
ci: verify coverage
