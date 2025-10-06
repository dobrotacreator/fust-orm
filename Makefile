pytest = uv run pytest
maturin = uv run maturin
mypy = uv run mypy
pyright = uv run pyright
ruff = uv run ruff


.PHONY: dev
dev:
	$(maturin) develop --uv


.PHONY: tests
tests: dev
	$(pytest) --no-header --no-cov -vv tests


.PHONY: tests-debug
tests-debug: dev
	$(pytest) --no-header --no-cov -vv tests --log-cli-level=DEBUG


.PHONY: py-format
py-format:
	$(ruff) format .
	$(ruff) check --fix .


.PHONY: py-lint
py-lint:
	$(ruff) check . --preview
	$(mypy) ./fust_orm ./tests
	$(pyright)
