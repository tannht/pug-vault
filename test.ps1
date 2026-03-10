# 🐶 PugVault Test Runner Script for Windows
# PowerShell version

param(
    [switch]$SkipBenchmarks
)

Write-Host "🐶 PugVault Test Suite Runner" -ForegroundColor Blue
Write-Host "==================================" -ForegroundColor Blue

# Function to run test with timing
function Run-Test {
    param(
        [string]$TestName,
        [string]$TestCommand
    )
    
    Write-Host "`n📋 Running $TestName..." -ForegroundColor Yellow
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    
    try {
        Invoke-Expression $TestCommand
        $stopwatch.Stop()
        $duration = $stopwatch.Elapsed.TotalSeconds
        Write-Host "✅ $TestName passed ($([math]::Round($duration, 1))s)" -ForegroundColor Green
        return $true
    }
    catch {
        $stopwatch.Stop()
        $duration = $stopwatch.Elapsed.TotalSeconds
        Write-Host "❌ $TestName failed ($([math]::Round($duration, 1))s)" -ForegroundColor Red
        return $false
    }
}

# Set test password for integration tests
$env:PUG_MASTER_PASSWORD = "test_password_for_local_testing"

$failedTests = 0
$totalTests = 0

# Code Quality Checks
Write-Host "`n🔍 Code Quality Checks" -ForegroundColor Blue
Write-Host "----------------------" -ForegroundColor Blue

if (Run-Test "Format Check" "cargo fmt --all -- --check") {
    $totalTests++
} else {
    $failedTests++
    $totalTests++
}

if (Run-Test "Clippy Lints" "cargo clippy --all-targets --all-features -- -D warnings") {
    $totalTests++
} else {
    $failedTests++
    $totalTests++
}

# Core Test Suites  
Write-Host "`n🧪 Core Test Suites" -ForegroundColor Blue
Write-Host "-------------------" -ForegroundColor Blue

if (Run-Test "Unit Tests" "cargo test --lib") {
    $totalTests++
} else {
    $failedTests++
    $totalTests++
}

if (Run-Test "Doc Tests" "cargo test --doc") {
    $totalTests++
} else {
    $failedTests++
    $totalTests++
}

if (Run-Test "Integration Tests" "cargo test --test integration_tests") {
    $totalTests++
} else {
    $failedTests++
    $totalTests++
}

if (Run-Test "Security Tests" "cargo test --test security_tests") {
    $totalTests++
} else {
    $failedTests++
    $totalTests++
}

# Optional benchmark tests
if (-not $SkipBenchmarks) {
    Write-Host "`n⚡ Performance Tests" -ForegroundColor Blue
    Write-Host "-------------------" -ForegroundColor Blue
    
    if (Run-Test "Benchmark Tests" "cargo test --test benchmark_tests") {
        $totalTests++
    } else {
        $failedTests++
        $totalTests++
        Write-Host "💡 Note: Benchmark tests may fail on slower machines" -ForegroundColor Yellow
    }
} else {
    Write-Host "`n⚡ Skipping benchmark tests (-SkipBenchmarks)" -ForegroundColor Yellow
}

# Build test
Write-Host "`n🔨 Build Tests" -ForegroundColor Blue  
Write-Host "---------------" -ForegroundColor Blue

if (Run-Test "Debug Build" "cargo build") {
    $totalTests++
} else {
    $failedTests++
    $totalTests++
}

if (Run-Test "Release Build" "cargo build --release") {
    $totalTests++
} else {
    $failedTests++
    $totalTests++
}

# Summary
Write-Host "`n📊 Test Summary" -ForegroundColor Blue
Write-Host "===============" -ForegroundColor Blue

$passedTests = $totalTests - $failedTests

if ($failedTests -eq 0) {
    Write-Host "🎉 All tests passed! ($passedTests/$totalTests)" -ForegroundColor Green
    Write-Host "✨ Your code is ready for commit! Gâu gâu! 🐶" -ForegroundColor Green
    exit 0
} else {
    Write-Host "💥 $failedTests/$totalTests test(s) failed" -ForegroundColor Red
    Write-Host "🔧 Please fix the failing tests before committing" -ForegroundColor Red
    exit 1
}