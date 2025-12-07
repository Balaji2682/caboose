#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

# One-click demo environment for Caboose that exercises all views with synthetic data.
# Usage: scripts/run_demo.sh [demo_dir]
# If demo_dir is omitted, /tmp/caboose-demo is used.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_DIR="${1:-/tmp/caboose-demo}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-"$ROOT_DIR/target"}"
TMPDIR_OVERRIDE="$CARGO_TARGET_DIR/tmp"

rm -rf "$DEMO_DIR"
echo "Creating demo environment at: $DEMO_DIR"
mkdir -p "$DEMO_DIR"/{config,frontend,scripts}
mkdir -p "$CARGO_TARGET_DIR" "$TMPDIR_OVERRIDE"

# Rails markers (auto-detect Rails + DB + background job + asset pipeline)
cat > "$DEMO_DIR/Gemfile" <<'EOF'
gem 'rails'
gem 'sidekiq'
gem 'vite_rails'
EOF

cat > "$DEMO_DIR/config/application.rb" <<'EOF'
module DemoApp; end
EOF

cat > "$DEMO_DIR/config/database.yml" <<'EOF'
default: { adapter: postgresql }
EOF

# Frontend markers (auto-detect frontend + package manager)
cat > "$DEMO_DIR/frontend/package.json" <<'EOF'
{ "name": "demo-ui", "version": "0.0.1", "dependencies": { "react-scripts": "5.0.0" } }
EOF
touch "$DEMO_DIR/frontend/vite.config.js"
touch "$DEMO_DIR/frontend/package-lock.json"

# Rails log generator (includes N+1 and various requests + heartbeat)
cat > "$DEMO_DIR/scripts/rails_logs.sh" <<'EOF'
#!/usr/bin/env bash
trap '' PIPE

heartbeat() {
  echo "[web] heartbeat: $(date +%H:%M:%S)"
}

# Initial heartbeat so Caboose shows logs immediately
heartbeat
sleep 0.2

echo "=== Request 1: Simple query ==="
echo "Started GET \"/users/123\" for 127.0.0.1 at 2024-01-01 12:00:00"
sleep 0.2
echo "Processing by UsersController#show as HTML"
sleep 0.2
echo "  User Load (0.5ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 123 LIMIT 1"
sleep 0.2
echo "Completed 200 OK in 15.7ms (Views: 8.4ms | ActiveRecord: 2.1ms)"
sleep 0.5

echo "=== Request 2: N+1 Query Example ==="
echo "Started GET \"/posts\" for 127.0.0.1 at 2024-01-01 12:00:05"
sleep 0.2
echo "Processing by PostsController#index as HTML"
sleep 0.2
echo "  Post Load (1.2ms)  SELECT \"posts\".* FROM \"posts\" ORDER BY created_at DESC LIMIT 10"
for i in {1..7}; do
  sleep 0.15
  echo "  User Load (0.4ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = $i LIMIT 1"
done
echo "Completed 200 OK in 85.3ms (Views: 45.2ms | ActiveRecord: 38.1ms)"
sleep 1

echo "=== Request 3: Another request ==="
echo "Started POST \"/articles\" for 127.0.0.1 at 2024-01-01 12:00:10"
sleep 0.2
echo "Processing by ArticlesController#create as HTML"
sleep 0.1
echo "  BEGIN (0.1ms)"
sleep 0.1
echo "  INSERT INTO \"articles\" (\"title\", \"body\", \"created_at\") VALUES ('Test', 'Content', NOW()) (1.2ms)"
sleep 0.2
echo "  COMMIT (0.3ms)"
sleep 0.1
echo "Completed 201 Created in 23.5ms (ActiveRecord: 5.6ms)"
sleep 1

echo "=== Request 4: More N+1 (comments) ==="
echo "Started GET \"/articles/5\" for 127.0.0.1"
sleep 0.2
echo "Processing by ArticlesController#show as HTML"
sleep 0.2
echo "  Article Load (0.6ms)  SELECT \"articles\".* FROM \"articles\" WHERE \"articles\".\"id\" = 5 LIMIT 1"
sleep 0.2
echo "  Comment Load (1.1ms)  SELECT \"comments\".* FROM \"comments\" WHERE \"comments\".\"article_id\" = 5"
for i in {10..13}; do
  sleep 0.15
  echo "  User Load (0.3ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = $i LIMIT 1"
done
echo "Completed 200 OK in 45.8ms (Views: 25.3ms | ActiveRecord: 18.5ms)"
sleep 1

# Continuous heartbeat so logs keep streaming
while true; do
  heartbeat
  echo "Started GET \"/api/health\" for 127.0.0.1"
  echo "Completed 200 OK in 2.3ms"
  sleep 2
done
EOF
chmod +x "$DEMO_DIR/scripts/rails_logs.sh"

# Exception generator
cat > "$DEMO_DIR/scripts/exceptions.sh" <<'EOF'
#!/usr/bin/env bash
trap '' PIPE
while true; do
  echo "NameError: undefined local variable or method \`user_123'"
  echo "  app/controllers/users_controller.rb:42:in \`show'"
  echo "done"
  sleep 5
done
EOF
chmod +x "$DEMO_DIR/scripts/exceptions.sh"

# Test + debugger output generator
cat > "$DEMO_DIR/scripts/tests.sh" <<'EOF'
#!/usr/bin/env bash
trap '' PIPE
while true; do
  echo "Minitest"
  echo "Finished in 0.123s"
  echo "1 runs, 2 assertions, 1 failures, 0 errors, 0 skips"
  echo "From: /app/foo.rb:42 [byebug]"
  sleep 8
done
EOF
chmod +x "$DEMO_DIR/scripts/tests.sh"

# Frontend dev server placeholder
cat > "$DEMO_DIR/scripts/frontend.sh" <<'EOF'
#!/usr/bin/env bash
trap '' PIPE
while true; do
  echo "vite dev server output"
  sleep 10
done
EOF
chmod +x "$DEMO_DIR/scripts/frontend.sh"

# Procfile wiring all demo processes
cat > "$DEMO_DIR/Procfile" <<'EOF'
web: stdbuf -oL -eL bash scripts/rails_logs.sh
worker: stdbuf -oL -eL bash scripts/exceptions.sh
tests: stdbuf -oL -eL bash scripts/tests.sh
frontend: stdbuf -oL -eL bash scripts/frontend.sh
EOF

echo "Demo setup complete."
echo "Starting Caboose TUI with demo data..."
cd "$DEMO_DIR"

BIN="$CARGO_TARGET_DIR/debug/caboose"
if [ ! -x "$BIN" ]; then
  echo "Build artifact not found at $BIN. Please run:"
  echo "  CARGO_TARGET_DIR=$CARGO_TARGET_DIR CARGO_INCREMENTAL=0 cargo build --manifest-path $ROOT_DIR/Cargo.toml"
  exit 1
fi

NO_PTY=1 TMPDIR="$TMPDIR_OVERRIDE" CARGO_TARGET_DIR="$CARGO_TARGET_DIR" "$BIN"
