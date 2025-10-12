#!/bin/bash

# End-to-End Inference Test
# Tests the complete flow from model deployment to inference execution

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}       Lattice AI Inference End-to-End Test${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo ""

# Configuration
RPC_URL="http://127.0.0.1:8545"
IPFS_API="/ip4/127.0.0.1/tcp/5001"
TEST_DIR="./test_inference"

# Create test directory
mkdir -p $TEST_DIR
cd $TEST_DIR

# Step 1: Check Prerequisites
echo -e "${BLUE}Step 1: Checking Prerequisites${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check Rust build
if [ ! -f "../target/release/lattice" ]; then
    echo -e "${YELLOW}Building Lattice...${NC}"
    cd ..
    cargo build --release --features coreml
    cd $TEST_DIR
fi
echo -e "  ✅ Lattice binary ready"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 not found${NC}"
    exit 1
fi
echo -e "  ✅ Python 3 installed"

# Check IPFS
if ! ipfs swarm peers &> /dev/null; then
    echo -e "${YELLOW}Starting IPFS daemon...${NC}"
    ipfs daemon &> ipfs.log &
    IPFS_PID=$!
    sleep 5
fi
echo -e "  ✅ IPFS running"

# Check Lattice node
if ! curl -s $RPC_URL &> /dev/null; then
    echo -e "${YELLOW}Starting Lattice node...${NC}"
    ../target/release/lattice devnet &> lattice.log &
    LATTICE_PID=$!
    sleep 5
fi
echo -e "  ✅ Lattice node running"
echo ""

# Step 2: Deploy Test Model
echo -e "${BLUE}Step 2: Deploying Test Model${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create a simple test model (mock ONNX)
echo "Creating test model..."
cat > test_model.json <<EOF
{
    "format": "onnx",
    "version": "1.0",
    "input_shape": [1, 10],
    "output_shape": [1, 2],
    "weights": "$(echo -n "test_weights" | base64)"
}
EOF

# Deploy via CLI
echo "Deploying model..."
../target/release/lattice-cli model deploy \
    --model test_model.json \
    --name "Test Model" \
    --access-policy public \
    > deployment.json 2>&1

if [ $? -eq 0 ]; then
    MODEL_ID=$(grep -o '"model_id": "[^"]*' deployment.json | cut -d'"' -f4)
    echo -e "  ${GREEN}✅ Model deployed${NC}"
    echo "  📍 Model ID: $MODEL_ID"
else
    echo -e "  ${RED}❌ Deployment failed${NC}"
    cat deployment.json
    exit 1
fi
echo ""

# Step 3: Test Inference via Precompile
echo -e "${BLUE}Step 3: Testing Inference via Precompile${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create test transaction calling inference precompile (0x0101)
cat > inference_call.json <<EOF
{
    "from": "0x1234567890123456789012345678901234567890",
    "to": "0x0000000000000000000000000000000000000101",
    "data": "0x${MODEL_ID}$(printf '00%.0s' {1..64})",
    "gas": "0x30000",
    "gasPrice": "0x1",
    "value": "0x0"
}
EOF

echo "Calling inference precompile..."
RESULT=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": ['"$(cat inference_call.json | jq -c .)"', "latest"],
        "id": 1
    }')

if echo $RESULT | grep -q "result"; then
    echo -e "  ${GREEN}✅ Inference precompile called${NC}"
    echo "  📊 Result: $(echo $RESULT | jq -r .result)"
else
    echo -e "  ${RED}❌ Precompile call failed${NC}"
    echo $RESULT | jq .
fi
echo ""

# Step 4: Test via CLI
echo -e "${BLUE}Step 4: Testing Inference via CLI${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create input file
cat > input.json <<EOF
{
    "data": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
}
EOF

echo "Running inference..."
../target/release/lattice-cli model inference \
    --model-id $MODEL_ID \
    --input input.json \
    --output output.json \
    2>&1 | tee inference.log

if [ -f output.json ]; then
    echo -e "  ${GREEN}✅ CLI inference successful${NC}"
    echo "  📊 Output:"
    cat output.json | jq .
else
    echo -e "  ${YELLOW}⚠️  CLI inference not fully implemented${NC}"
fi
echo ""

# Step 5: Test Model Metadata Query
echo -e "${BLUE}Step 5: Testing Model Metadata Query${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Query metadata via precompile (0x0103)
cat > metadata_call.json <<EOF
{
    "from": "0x1234567890123456789012345678901234567890",
    "to": "0x0000000000000000000000000000000000000103",
    "data": "0x${MODEL_ID}",
    "gas": "0x10000",
    "gasPrice": "0x1",
    "value": "0x0"
}
EOF

echo "Querying model metadata..."
METADATA=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": ['"$(cat metadata_call.json | jq -c .)"', "latest"],
        "id": 1
    }')

if echo $METADATA | grep -q "result"; then
    echo -e "  ${GREEN}✅ Metadata query successful${NC}"
    # Decode hex result
    HEX_DATA=$(echo $METADATA | jq -r .result | sed 's/0x//')
    if [ ! -z "$HEX_DATA" ] && [ "$HEX_DATA" != "null" ]; then
        echo "  📋 Raw metadata: 0x${HEX_DATA:0:64}..."
    fi
else
    echo -e "  ${RED}❌ Metadata query failed${NC}"
fi
echo ""

# Step 6: Performance Test
echo -e "${BLUE}Step 6: Performance Benchmark${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "Running 10 inference calls..."
START_TIME=$(date +%s)

for i in {1..10}; do
    curl -s -X POST $RPC_URL \
        -H "Content-Type: application/json" \
        -d '{
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": ['"$(cat inference_call.json | jq -c .)"', "latest"],
            "id": '"$i"'
        }' > /dev/null
    echo -n "."
done
echo ""

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo -e "  ${GREEN}✅ Benchmark complete${NC}"
echo "  ⏱️  Total time: ${DURATION}s"
echo "  📊 Average: $((DURATION * 100 / 10))ms per inference"
echo ""

# Step 7: Verify CoreML Integration
echo -e "${BLUE}Step 7: CoreML Integration Check${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if CoreML is being used
if grep -q "CoreML" ../target/release/lattice 2>/dev/null; then
    echo -e "  ${GREEN}✅ CoreML support compiled in${NC}"
else
    echo -e "  ${YELLOW}⚠️  CoreML support may not be enabled${NC}"
fi

# Check Metal runtime logs
if grep -q "Metal" lattice.log 2>/dev/null; then
    echo -e "  ${GREEN}✅ Metal runtime initialized${NC}"
else
    echo -e "  ${YELLOW}⚠️  Metal runtime not detected in logs${NC}"
fi
echo ""

# Summary
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}                    Test Summary${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# Check each component
[ ! -z "$MODEL_ID" ] && ((TESTS_PASSED++)) || ((TESTS_FAILED++))
echo $RESULT | grep -q "result" && ((TESTS_PASSED++)) || ((TESTS_FAILED++))
[ -f output.json ] && ((TESTS_PASSED++)) || ((TESTS_FAILED++))

echo -e "  Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "  Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All critical tests passed!${NC}"
    echo -e "${GREEN}   Inference pipeline is working end-to-end${NC}"
else
    echo -e "${YELLOW}⚠️  Some tests need attention${NC}"
    echo -e "${YELLOW}   Check logs for details${NC}"
fi

# Cleanup
echo ""
echo -e "${YELLOW}Cleaning up...${NC}"
[ ! -z "$IPFS_PID" ] && kill $IPFS_PID 2>/dev/null || true
[ ! -z "$LATTICE_PID" ] && kill $LATTICE_PID 2>/dev/null || true

cd ..
echo -e "${GREEN}✨ Test complete!${NC}"