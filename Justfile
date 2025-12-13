precommit:
    cargo fmt
    cargo clippy -- -D warnings
    cargo clippy --features padding_api -- -D warnings
    cargo test --features padding_api

coverage:
    cargo llvm-cov --features padding_api --lcov --output-path=lcov.info --ignore-filename-regex tests\.rs
    genhtml lcov.info --dark-mode --flat --missed --output-directory target/coverage_html
