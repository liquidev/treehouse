port := "8080"

serve:
    cargo watch -- cargo run -- serve --port {{port}}

fix:
    cargo run -- fix-all --apply

ulid:
    cargo run -- ulid

deploy:
    bash admin/deploy.bash
