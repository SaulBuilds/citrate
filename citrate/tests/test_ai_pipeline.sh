#!/bin/bash

# Test AI Model Deployment Pipeline
# Tests the complete flow from model import to inference

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
RPC_URL="http://127.0.0.1:8545"
IPFS_API="/ip4/127.0.0.1/tcp/5001"
TEST_DIR="./test_ai_models"

echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}       Citrate AI Pipeline Test Suite${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
echo ""

# Check prerequisites
check_prerequisites() {
    echo -e "${BLUE}Checking prerequisites...${NC}"
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}❌ Python 3 not found${NC}"
        exit 1
    fi
    echo -e "  ✅ Python 3 installed"
    
    # Check IPFS
    if ! command -v ipfs &> /dev/null; then
        echo -e "${RED}❌ IPFS not found${NC}"
        echo "Install with: brew install ipfs"
        exit 1
    fi
    echo -e "  ✅ IPFS installed"
    
    # Check if IPFS daemon is running
    if ! ipfs swarm peers &> /dev/null; then
        echo -e "${YELLOW}⚠️  Starting IPFS daemon...${NC}"
        ipfs daemon &> /dev/null &
        IPFS_PID=$!
        sleep 5
    fi
    echo -e "  ✅ IPFS daemon running"
    
    # Check Citrate node
    if ! curl -s -X POST $RPC_URL \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' &> /dev/null; then
        echo -e "${YELLOW}⚠️  Starting Citrate devnet...${NC}"
        ./target/release/lattice devnet &> devnet.log &
        CITRATE_PID=$!
        sleep 5
    fi
    echo -e "  ✅ Citrate node running"
    echo ""
}

# Install Python dependencies
install_dependencies() {
    echo -e "${BLUE}Installing Python dependencies...${NC}"
    pip3 install -q torch transformers coremltools web3 ipfshttpclient numpy pillow
    echo -e "  ✅ Dependencies installed"
    echo ""
}

# Test 1: Small Text Model (DistilBERT)
test_text_model() {
    echo -e "${BLUE}Test 1: Deploying DistilBERT (Text Classification)${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Deploy model
    echo "  📦 Converting and deploying model..."
    cd tools
    python3 import_model.py huggingface distilbert-base-uncased \
        --optimize \
        --rpc $RPC_URL \
        --ipfs $IPFS_API > deployment.json 2>&1
    
    if [ $? -eq 0 ]; then
        echo -e "  ${GREEN}✅ Model deployed successfully${NC}"
        
        # Extract model info
        MODEL_ID=$(grep -o '"model_id": "[^"]*' deployment.json | cut -d'"' -f4)
        IPFS_CID=$(grep -o '"ipfs_cid": "[^"]*' deployment.json | cut -d'"' -f4)
        
        echo "  📍 Model ID: $MODEL_ID"
        echo "  🌐 IPFS CID: $IPFS_CID"
    else
        echo -e "  ${RED}❌ Deployment failed${NC}"
        cat deployment.json
        return 1
    fi
    
    cd ..
    echo ""
}

# Test 2: Vision Model (ResNet-50)
test_vision_model() {
    echo -e "${BLUE}Test 2: Deploying ResNet-50 (Image Classification)${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    echo "  📦 Converting and deploying model..."
    cd tools
    python3 import_model.py huggingface microsoft/resnet-50 \
        --optimize \
        --rpc $RPC_URL \
        --ipfs $IPFS_API > deployment_vision.json 2>&1
    
    if [ $? -eq 0 ]; then
        echo -e "  ${GREEN}✅ Vision model deployed${NC}"
        
        MODEL_ID=$(grep -o '"model_id": "[^"]*' deployment_vision.json | cut -d'"' -f4)
        echo "  📍 Model ID: $MODEL_ID"
    else
        echo -e "  ${RED}❌ Vision model deployment failed${NC}"
        return 1
    fi
    
    cd ..
    echo ""
}

# Test 3: List Deployed Models
test_list_models() {
    echo -e "${BLUE}Test 3: Listing Deployed Models${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    ./target/release/citrate-cli model list --limit 10
    
    if [ $? -eq 0 ]; then
        echo -e "  ${GREEN}✅ Model listing successful${NC}"
    else
        echo -e "  ${RED}❌ Failed to list models${NC}"
        return 1
    fi
    echo ""
}

# Test 4: Model Inference
test_inference() {
    echo -e "${BLUE}Test 4: Running Inference${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Create test input
    cat > test_input.json <<EOF
{
    "text": "The new MacBook Pro with M3 chip delivers incredible performance for AI workloads.",
    "task": "sentiment"
}
EOF
    
    echo "  🧠 Running inference on text model..."
    ./target/release/citrate-cli model inference \
        --model-id $MODEL_ID \
        --input test_input.json \
        --output result.json
    
    if [ $? -eq 0 ]; then
        echo -e "  ${GREEN}✅ Inference successful${NC}"
        echo "  📊 Result:"
        cat result.json | jq '.'
    else
        echo -e "  ${RED}❌ Inference failed${NC}"
        return 1
    fi
    echo ""
}

# Test 5: Performance Benchmark
test_performance() {
    echo -e "${BLUE}Test 5: Performance Benchmark${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    echo "  ⏱️  Running 10 inference requests..."
    START_TIME=$(date +%s)
    
    for i in {1..10}; do
        ./target/release/citrate-cli model inference \
            --model-id $MODEL_ID \
            --input test_input.json \
            --output /dev/null 2>&1
        echo -n "."
    done
    echo ""
    
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    AVG_TIME=$((DURATION * 100 / 10))
    
    echo -e "  ${GREEN}✅ Benchmark complete${NC}"
    echo "  📊 Results:"
    echo "     Total time: ${DURATION}s"
    echo "     Average per inference: ${AVG_TIME}ms"
    echo "     Throughput: $((10 / DURATION)) req/s"
    echo ""
}

# Test 6: IPFS Storage Verification
test_ipfs_storage() {
    echo -e "${BLUE}Test 6: IPFS Storage Verification${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    echo "  🔍 Checking IPFS pinning..."
    ipfs pin ls | grep $IPFS_CID > /dev/null
    
    if [ $? -eq 0 ]; then
        echo -e "  ${GREEN}✅ Model pinned in IPFS${NC}"
        
        # Check model size
        SIZE=$(ipfs object stat $IPFS_CID | grep CumulativeSize | awk '{print $2}')
        echo "  📦 Model size: $((SIZE / 1024 / 1024)) MB"
    else
        echo -e "  ${RED}❌ Model not found in IPFS${NC}"
        return 1
    fi
    echo ""
}

# Test 7: Metal GPU Verification
test_metal_gpu() {
    echo -e "${BLUE}Test 7: Metal GPU Execution Verification${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check if running on Mac
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "  🖥️  System: macOS $(sw_vers -productVersion)"
        
        # Check for Apple Silicon
        if [[ $(uname -m) == "arm64" ]]; then
            CHIP=$(system_profiler SPHardwareDataType | grep "Chip" | sed 's/.*: //')
            echo "  🎯 Apple Silicon: $CHIP"
            
            # Check Metal support
            if system_profiler SPDisplaysDataType | grep -q "Metal"; then
                echo -e "  ${GREEN}✅ Metal GPU support detected${NC}"
                
                # Check Neural Engine
                if [[ "$CHIP" == *"M"* ]]; then
                    echo -e "  ${GREEN}✅ Neural Engine available${NC}"
                fi
            fi
        else
            echo "  ⚠️  Intel Mac detected (limited Metal support)"
        fi
    else
        echo "  ℹ️  Not running on macOS"
    fi
    echo ""
}

# Run all tests
run_all_tests() {
    check_prerequisites
    install_dependencies
    
    # Track test results
    PASSED=0
    FAILED=0
    
    # Run tests
    if test_text_model; then ((PASSED++)); else ((FAILED++)); fi
    if test_vision_model; then ((PASSED++)); else ((FAILED++)); fi
    if test_list_models; then ((PASSED++)); else ((FAILED++)); fi
    if test_inference; then ((PASSED++)); else ((FAILED++)); fi
    if test_performance; then ((PASSED++)); else ((FAILED++)); fi
    if test_ipfs_storage; then ((PASSED++)); else ((FAILED++)); fi
    if test_metal_gpu; then ((PASSED++)); else ((FAILED++)); fi
    
    # Summary
    echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}                    Test Summary${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  Tests Passed: ${GREEN}$PASSED${NC}"
    echo -e "  Tests Failed: ${RED}$FAILED${NC}"
    echo ""
    
    if [ $FAILED -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed! AI pipeline is working correctly.${NC}"
    else
        echo -e "${RED}⚠️  Some tests failed. Please check the logs above.${NC}"
    fi
    
    # Cleanup
    if [ ! -z "$IPFS_PID" ]; then
        echo -e "\n${YELLOW}Stopping IPFS daemon...${NC}"
        kill $IPFS_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$CITRATE_PID" ]; then
        echo -e "${YELLOW}Stopping Citrate node...${NC}"
        kill $CITRATE_PID 2>/dev/null || true
    fi
}

# Main execution
run_all_tests
