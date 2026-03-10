#!/bin/bash

# 🐶 PugVault Test Runner Script
set -e

# Colors for output 
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m' 
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🐶 PugVault Test Suite Runner${NC}"
echo "=================================="

# Function to run test with timing
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${YELLOW}📋 Running $test_name...${NC}"
    start_time=$(date +%s)
    
    if eval "$test_command"; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${GREEN}✅ $test_name passed (${duration}s)${NC}"
        return 0
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${RED}❌ $test_name failed (${duration}s)${NC}"
        return 1
    fi
}

# Export test password for integration tests
export PUG_MASTER_PASSWORD="test_password_for_local_testing"

failed_tests=0
total_tests=0

# Code Quality Checks
echo -e "\n${BLUE}🔍 Code Quality Checks${NC}"
echo "----------------------"

if run_test "Format Check" "cargo fmt --all -- --check"; then
    ((total_tests++))
else
    ((failed_tests++))
    ((total_tests++))
fi

if run_test "Clippy Lints" "cargo clippy --all-targets --all-features -- -D warnings"; then
    ((total_tests++))
else
    ((failed_tests++))
    ((total_tests++))
fi

# Core Test Suites
echo -e "\n${BLUE}🧪 Core Test Suites${NC}"  
echo "-------------------"

if run_test "Unit Tests" "cargo test --lib"; then
    ((total_tests++))
else
    ((failed_tests++))
    ((total_tests++))
fi

if run_test "Doc Tests" "cargo test --doc"; then
    ((total_tests++))
else
    ((failed_tests++))
    ((total_tests++))
fi

if run_test "Integration Tests" "cargo test --test integration_tests"; then
    ((total_tests++))
else
    ((failed_tests++))
    ((total_tests++))
fi

if run_test "Security Tests" "cargo test --test security_tests"; then
    ((total_tests++))
else
    ((failed_tests++))
    ((total_tests++))
fi

# Optional benchmark tests (can be slow)
if [[ "${1:-}" != "--skip-benchmarks" ]]; then
    echo -e "\n${BLUE}⚡ Performance Tests${NC}"
    echo "-------------------"
    
    if run_test "Benchmark Tests" "cargo test --test benchmark_tests"; then
        ((total_tests++))
    else
        ((failed_tests++))
        ((total_tests++))
        echo -e "${YELLOW}💡 Note: Benchmark tests may fail on slower machines${NC}"
    fi
else
    echo -e "\n${YELLOW}⚡ Skipping benchmark tests (--skip-benchmarks)${NC}"
fi

# Build test
echo -e "\n${BLUE}🔨 Build Tests${NC}"
echo "---------------"

if run_test "Debug Build" "cargo build"; then
    ((total_tests++))
else
    ((failed_tests++))
    ((total_tests++))
fi

if run_test "Release Build" "cargo build --release"; then
    ((total_tests++))
else
    ((failed_tests++))
    ((total_tests++))
fi

# Summary
echo -e "\n${BLUE}📊 Test Summary${NC}"
echo "==============="

passed_tests=$((total_tests - failed_tests))

if [ $failed_tests -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! ($passed_tests/$total_tests)${NC}"
    echo -e "${GREEN}✨ Your code is ready for commit! Gâu gâu! 🐶${NC}"
    exit 0
else
    echo -e "${RED}💥 $failed_tests/$total_tests test(s) failed${NC}"
    echo -e "${RED}🔧 Please fix the failing tests before committing${NC}"
    exit 1
fi