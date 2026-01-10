# Makefile for nptdms benchmark suite
# 
# Usage:
#   make setup     - Install dependencies and validate setup
#   make smoke     - Run quick smoke tests (CI-safe)
#   make full      - Run comprehensive benchmark suite
#   make clean     - Clean generated files
#   make help      - Show this help

.PHONY: help setup smoke full clean test-files results

# Default target
help:
	@echo "nptdms Benchmark Suite"
	@echo "======================"
	@echo ""
	@echo "Available targets:"
	@echo "  setup      - Install dependencies and validate setup"
	@echo "  smoke      - Run quick smoke tests (CI-safe, ~5 minutes)"
	@echo "  full       - Run comprehensive benchmark suite (~30-60 minutes)"
	@echo "  test-files - Generate benchmark test files"
	@echo "  clean      - Clean generated files and results"
	@echo "  results    - Show latest benchmark results"
	@echo "  compare    - Compare latest two benchmark runs"
	@echo "  help       - Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make setup && make smoke    # Setup and run quick tests"
	@echo "  make full                   # Run comprehensive benchmarks"
	@echo "  make clean && make smoke    # Clean start with smoke tests"

# Setup and validation
setup:
	@echo "Setting up nptdms benchmark suite..."
	python -m pip install --upgrade pip
	python -m pip install -r requirements.txt
	python setup_benchmarks.py

# Quick smoke tests (CI-safe)
smoke:
	@echo "Running smoke tests (quick, CI-safe)..."
	python run_benchmarks.py --mode smoke --format both
	@echo "Smoke tests complete! Check results/ directory."

# Comprehensive benchmark suite
full:
	@echo "Running full benchmark suite (this may take 30-60 minutes)..."
	python run_benchmarks.py --mode full --format both
	@echo "Full benchmarks complete! Check results/ directory."

# Generate test files only
test-files:
	@echo "Generating benchmark test files..."
	python generate_test_files.py full
	@echo "Test files generated in test_files/ directory."

# Show latest results
results:
	@echo "Latest benchmark results:"
	@python -c "
import csv
from pathlib import Path
import os

results_dir = Path('results')
if not results_dir.exists():
    print('No results directory found. Run benchmarks first.')
    exit(1)

csv_files = list(results_dir.glob('*.csv'))
if not csv_files:
    print('No CSV result files found. Run benchmarks first.')
    exit(1)

latest_file = max(csv_files, key=lambda p: p.stat().st_mtime)
print(f'File: {latest_file}')
print(f'Size: {latest_file.stat().st_size} bytes')
print(f'Modified: {latest_file.stat().st_mtime}')
print()

with open(latest_file, 'r') as f:
    reader = csv.DictReader(f)
    results = list(reader)

print(f'Total benchmarks: {len(results)}')

# Group by category
categories = {}
for r in results:
    cat = r['benchmark_name'].split('_')[0]
    if cat not in categories:
        categories[cat] = []
    categories[cat].append(float(r['time_sec']))

for category, times in categories.items():
    avg_time = sum(times) / len(times)
    print(f'{category}: {len(times)} tests, avg {avg_time:.3f}s')
"

# Compare two latest benchmark runs
compare:
	@echo "Comparing latest benchmark runs..."
	@python example_usage.py

# Clean generated files
clean:
	@echo "Cleaning generated files..."
	rm -rf test_files/
	rm -rf results/
	rm -rf __pycache__/
	rm -rf *.pyc
	find . -name "*.pyc" -delete 2>/dev/null || true
	find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
	@echo "Clean complete."

# Clean only large test files (keep small ones for quick testing)
clean-large:
	@echo "Cleaning large test files (>50MB)..."
	@python -c "
from pathlib import Path
test_files_dir = Path('test_files')
if test_files_dir.exists():
    large_files = []
    for f in test_files_dir.glob('*.tdms'):
        size_mb = f.stat().st_size / 1024 / 1024
        if size_mb > 50:
            large_files.append((f, size_mb))
    
    if large_files:
        for f, size in large_files:
            print(f'Removing {f.name}: {size:.1f} MB')
            f.unlink()
        print(f'Cleaned up {len(large_files)} large files')
    else:
        print('No large files found')
else:
    print('No test_files directory found')
"

# Validate benchmark setup
validate:
	@echo "Validating benchmark setup..."
	python -c "
import sys
print(f'Python: {sys.version}')

try:
    import nptdms
    print(f'nptdms: {nptdms.__version__}')
except ImportError:
    print('❌ nptdms not installed')
    exit(1)

try:
    import numpy
    print(f'numpy: {numpy.__version__}')
except ImportError:
    print('❌ numpy not installed')
    exit(1)

try:
    import psutil
    print(f'psutil: {psutil.__version__}')
except ImportError:
    print('❌ psutil not installed')
    exit(1)

print('✅ All dependencies available')
"

# Development targets
dev-smoke: clean setup smoke
dev-full: clean setup full

# CI targets (for GitHub Actions)
ci-smoke:
	python run_benchmarks.py --mode smoke --format json --no-summary

ci-full:
	python run_benchmarks.py --mode full --format json --no-summary