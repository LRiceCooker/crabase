dev:
    cargo tauri dev

set dotenv-load

build:
    cargo tauri build

ralph:
    ./ralph/ralph.sh

log:
    tail -f ralph/ralph.log | jq -Rr 'try (fromjson | if .type == "assistant" then .message.content[]? | if .type == "text" then "💬 \(.text)" elif .type == "tool_use" then "🔧 \(.name)(\(.input | keys | join(", ")))" else empty end elif .type == "user" then .message.content[]? | if .type == "tool_result" then (if .is_error then "❌ \(.content[:150])" else "✅ \(.content[:150])" end) else empty end else empty end) catch empty'

test-setup:
    ./tests/setup.sh

test-teardown:
    ./tests/teardown.sh

test:
    just test-frontend
    just test-e2e

test-frontend:
    npx vitest run

test-server:
    cargo run --manifest-path tests/test_server/Cargo.toml

test-e2e:
    npx playwright test --config tests/e2e/playwright.config.ts

test-e2e-headed:
    npx playwright test --config tests/e2e/playwright.config.ts --headed
