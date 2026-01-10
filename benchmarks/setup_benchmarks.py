#!/usr/bin/env python3
"""
Setup script for nptdms benchmark suite.

This script helps set up the benchmark environment and validates
that all dependencies are correctly installed.
"""

import sys
import subprocess
from pathlib import Path


def check_python_version():
    """Check Python version compatibility."""
    if sys.version_info < (3, 10):
        print("❌ Python 3.10 or higher is required")
        print(f"Current version: {sys.version}")
        return False
    
    print(f"✅ Python version: {sys.version.split()[0]}")
    return True


def install_dependencies():
    """Install required dependencies."""
    requirements_file = Path(__file__).parent / "requirements.txt"
    
    if not requirements_file.exists():
        print("❌ requirements.txt not found")
        return False
    
    print("Installing dependencies...")
    try:
        subprocess.check_call([
            sys.executable, "-m", "pip", "install", "-r", str(requirements_file)
        ])
        print("✅ Dependencies installed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to install dependencies: {e}")
        return False


def validate_imports():
    """Validate that all required modules can be imported."""
    required_modules = [
        ('nptdms', 'nptdms'),
        ('numpy', 'numpy'),
        ('psutil', 'psutil'),
    ]
    
    print("Validating imports...")
    all_good = True
    
    for module_name, import_name in required_modules:
        try:
            __import__(import_name)
            print(f"✅ {module_name}")
        except ImportError as e:
            print(f"❌ {module_name}: {e}")
            all_good = False
    
    return all_good


def create_directories():
    """Create necessary directories."""
    base_dir = Path(__file__).parent
    directories = [
        base_dir / "test_files",
        base_dir / "results"
    ]
    
    print("Creating directories...")
    for directory in directories:
        directory.mkdir(exist_ok=True)
        print(f"✅ {directory}")
    
    return True


def run_smoke_test():
    """Run a quick smoke test to validate setup."""
    print("Running smoke test...")
    
    try:
        # Import benchmark modules
        from benchmark_utils import BenchmarkResult, BenchmarkTimer
        from generate_test_files import main as generate_main
        
        # Test timer functionality
        timer = BenchmarkTimer()
        timer.start()
        
        # Small delay
        import time
        time.sleep(0.1)
        
        elapsed, memory = timer.stop()
        
        if elapsed > 0.05 and elapsed < 0.5:  # Should be around 0.1 seconds
            print("✅ Timing functionality works")
        else:
            print(f"⚠️  Timing seems off: {elapsed:.3f}s (expected ~0.1s)")
        
        print("✅ Smoke test passed")
        return True
        
    except Exception as e:
        print(f"❌ Smoke test failed: {e}")
        return False


def main():
    """Main setup function."""
    print("nptdms Benchmark Suite Setup")
    print("=" * 40)
    
    success = True
    
    # Check Python version
    if not check_python_version():
        success = False
    
    # Install dependencies
    if success and not install_dependencies():
        success = False
    
    # Validate imports
    if success and not validate_imports():
        success = False
    
    # Create directories
    if success and not create_directories():
        success = False
    
    # Run smoke test
    if success and not run_smoke_test():
        success = False
    
    print("\n" + "=" * 40)
    if success:
        print("✅ Setup completed successfully!")
        print("\nNext steps:")
        print("1. Run smoke tests: python run_benchmarks.py --mode smoke")
        print("2. Run full benchmarks: python run_benchmarks.py --mode full")
        print("3. Check results in the 'results/' directory")
    else:
        print("❌ Setup failed. Please fix the issues above.")
        sys.exit(1)


if __name__ == "__main__":
    main()