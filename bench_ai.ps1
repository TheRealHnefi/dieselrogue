<#
.SYNOPSIS
    Run the AI navigation benchmark (flow fields vs pure A*) with tunable params.

.DESCRIPTION
    Wraps `cargo test --release -- --ignored --nocapture ai_benchmark`, setting the
    BENCH_* environment variables the harness reads. By default it filters cargo's
    output down to just the header and the six result rows; pass -ShowAll to see the
    full output (map generation, field builds, etc.).

.EXAMPLE
    ./bench_ai.ps1                       # defaults: 2000 actors, 512x512, 60 ticks
    ./bench_ai.ps1 -Actors 4000 -Ticks 150
    ./bench_ai.ps1 -Size 24 -Parallel 0  # bigger map, serial AI
    ./bench_ai.ps1 -ShowAll              # unfiltered cargo output
#>
param(
    [int]$Size     = 16,   # map size in 32-tile blocks (16 => 512x512 tiles)
    [int]$Actors   = 2000, # target actor count
    [int]$Ticks    = 60,   # timed ticks per configuration
    [int]$Warmup   = 20,   # untimed warm-up ticks
    [int]$Parallel = 1,    # 1 = rayon parallel AI, 0 = serial
    [switch]$ShowAll       # show full cargo output instead of just results
)

$ErrorActionPreference = 'Stop'
Set-Location -Path $PSScriptRoot

$env:BENCH_SIZE     = $Size
$env:BENCH_ACTORS   = $Actors
$env:BENCH_TICKS    = $Ticks
$env:BENCH_WARMUP   = $Warmup
$env:BENCH_PARALLEL = $Parallel

Write-Host "Running AI benchmark: size=$Size actors=$Actors ticks=$Ticks warmup=$Warmup parallel=$Parallel" -ForegroundColor Cyan

$output = cargo test --release -- --ignored --nocapture ai_benchmark 2>&1

if ($ShowAll) {
    $output
} else {
    # Header line + the six "<scenario> fields=..." result rows.
    $output | Select-String -Pattern 'AI benchmark:|fields=(ON|OFF)'
}
