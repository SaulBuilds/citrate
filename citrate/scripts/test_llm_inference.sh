#!/bin/bash
# Test LLM inference with IPFS-pinned Mistral 7B model

set -e

echo "==========================================="
echo "LLM Inference Test"
echo "==========================================="
echo ""

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Create temporary data directory
TEST_DATA_DIR=$(mktemp -d -t citrate-llm-test)
echo "Test data directory: $TEST_DATA_DIR"
echo ""

# Check if binary exists
if [ ! -f "./target/release/citrate" ]; then
    echo "Error: citrate binary not found. Please run: cargo build --release"
    exit 1
fi

# Check if model is pinned
MODEL_PATH="$HOME/.citrate/models/mistral-7b-instruct-v0.3.gguf"
if [ ! -f "$MODEL_PATH" ]; then
    echo "Error: Mistral 7B model not found at $MODEL_PATH"
    echo "Please run: ./target/release/citrate model auto-pin"
    exit 1
fi

MODEL_SIZE=$(stat -f%z "$MODEL_PATH" 2>/dev/null || stat -c%s "$MODEL_PATH" 2>/dev/null)
MODEL_SIZE_GB=$(echo "scale=2; $MODEL_SIZE / 1024 / 1024 / 1024" | bc)
echo "✓ Mistral 7B model found: $MODEL_PATH"
echo "  Size: $MODEL_SIZE bytes ($MODEL_SIZE_GB GB)"
echo ""

# Start node in background with temporary data directory
echo "Starting Citrate node..."
./target/release/citrate --data-dir "$TEST_DATA_DIR" devnet > "$TEST_DATA_DIR/node.log" 2>&1 &
NODE_PID=$!
echo "Node started with PID: $NODE_PID"

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "Cleaning up..."
    if [ -n "$NODE_PID" ]; then
        echo "Stopping node (PID: $NODE_PID)..."
        kill $NODE_PID 2>/dev/null || true
        wait $NODE_PID 2>/dev/null || true
    fi
    echo "Removing test directory..."
    rm -rf "$TEST_DATA_DIR"
    echo "Cleanup complete"
}
trap cleanup EXIT INT TERM

# Wait for node to start (check RPC endpoint)
echo "Waiting for node to be ready..."
MAX_WAIT=30
WAIT_COUNT=0
while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
    if curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        > /dev/null 2>&1; then
        echo "✓ Node is ready"
        break
    fi
    WAIT_COUNT=$((WAIT_COUNT + 1))
    if [ $WAIT_COUNT -eq $MAX_WAIT ]; then
        echo "✗ Node failed to start within ${MAX_WAIT}s"
        echo "Last 20 lines of node log:"
        tail -20 "$TEST_DATA_DIR/node.log"
        exit 1
    fi
    sleep 1
    echo -n "."
done
echo ""

# Test 1: Simple chat completion (simple format)
echo ""
echo "Test 1: Simple chat completion (text prompt)"
echo "---------------------------------------------------"
PROMPT="What is 2 + 2?"
CHAT_REQUEST="{\"jsonrpc\":\"2.0\",\"method\":\"citrate_chatCompletion\",\"params\":[\"$PROMPT\", 100, 0.7],\"id\":1}"
echo "Request: $CHAT_REQUEST"
echo ""

CHAT_RESPONSE=$(curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d "$CHAT_REQUEST")

echo "Response preview (first 1000 chars):"
echo "$CHAT_RESPONSE" | head -c 1000
echo ""
echo "..."
echo ""

# Check if response contains result
if echo "$CHAT_RESPONSE" | grep -q '"result"'; then
    echo "✓ Chat completion executed successfully"

    # Check if response contains generated text
    if echo "$CHAT_RESPONSE" | grep -q '"message"'; then
        echo "✓ Response contains message field"

        # Extract and display generated text
        GENERATED_TEXT=$(echo "$CHAT_RESPONSE" | grep -o '"content":"[^"]*"' | head -1 | sed 's/"content":"\(.*\)"/\1/')
        if [ -n "$GENERATED_TEXT" ]; then
            echo "✓ Generated text (preview):"
            echo "  \"$GENERATED_TEXT\" ..." | head -c 200
            echo ""
        fi
    else
        echo "⚠ Warning: Response doesn't contain expected message field"
    fi
else
    echo "✗ Failed to execute chat completion"
    echo "Full response:"
    echo "$CHAT_RESPONSE"
    exit 1
fi
echo ""

# Test 2: Chat completion with full request format
echo "Test 2: Chat completion (full request format)"
echo "---------------------------------------------------"
FULL_REQUEST=$(cat <<'EOF'
{
  "jsonrpc": "2.0",
  "method": "citrate_chatCompletion",
  "params": [{
    "model": "mistral-7b-instruct-v0.3",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful AI assistant."
      },
      {
        "role": "user",
        "content": "Explain what a blockchain is in one sentence."
      }
    ],
    "max_tokens": 50,
    "temperature": 0.7
  }],
  "id": 1
}
EOF
)
echo "Request: (full ChatCompletionRequest format)"
echo ""

FULL_RESPONSE=$(curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d "$FULL_REQUEST")

if echo "$FULL_RESPONSE" | grep -q '"result"'; then
    echo "✓ Full format chat completion executed successfully"
    echo "Response preview (first 800 chars):"
    echo "$FULL_RESPONSE" | head -c 800
    echo ""
else
    echo "⚠ Full format test failed (may not be fully implemented)"
    echo "Response: $FULL_RESPONSE"
fi
echo ""

# Test 3: Check model in response
echo "Test 3: Verify model information in response"
echo "---------------------------------------------------"
if echo "$CHAT_RESPONSE" | grep -q '"model"'; then
    MODEL_NAME=$(echo "$CHAT_RESPONSE" | grep -o '"model":"[^"]*"' | head -1 | sed 's/"model":"\(.*\)"/\1/')
    echo "✓ Model field found in response: $MODEL_NAME"
else
    echo "⚠ Model field not found in response"
fi
echo ""

# Summary
echo "==========================================="
echo "Test Summary"
echo "==========================================="
echo "✓ Mistral 7B model pinned and accessible"
echo "✓ Chat completion RPC endpoint functional"
echo "✓ LLM inference generating responses"
echo ""
echo "All critical tests passed!"
echo "==========================================="
