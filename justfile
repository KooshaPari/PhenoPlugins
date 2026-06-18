# Phenotype-org shared justfile. Imported from phenotype-tooling/just/phenotype.just.
# To override a recipe locally, redefine it after the import.
import? "/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-tooling/just/phenotype.just"

# License + advisory + ban + source checks (cargo-deny)
# `audit` is already provided by the shared library; this adds a separate
# `deny` recipe so `just deny` is available across all Phenotype-org repos.
deny:
    cargo deny check

# Fleet-wide grading gate (uses vendored or central grade.sh)
grade:
    @if [ -f grade.sh ]; then ./grade.sh; \
    elif [ -f ../grade.sh ]; then bash ../grade.sh; \
    else echo "no grade.sh found (vendored or central)"; exit 1; \
    fi

grade-fast:
    @if [ -f grade.sh ]; then ./grade.sh --fast; \
    elif [ -f ../grade.sh ]; then bash ../grade.sh --fast; \
    else echo "no grade.sh found"; exit 1; \
    fi

# Measure code coverage (SSOT: see grade.sh for the canonical command)
coverage:
    cargo llvm-cov --workspace --fail-under-lines 85
