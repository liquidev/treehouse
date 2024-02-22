port := "8080"
export TREEHOUSE_SITE := "http://localhost:" + port

serve:
    cargo watch -- cargo run -- serve --port {{port}}

fix:
    cargo run -- fix-all --apply

ulid:
    cargo run -- ulid
