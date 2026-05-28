import os
import re
import csv
import sys
import time
import subprocess

# --- CONFIGURATION ---
# Define the widths and depths to test. 
# Based on your provided CSV, it tests even numbers up to 30.
WIDTHS = list(range(2, 32, 2))
DEPTHS = list(range(2, 32, 2))
ITERATIONS = 10

# Maximum time (in seconds) to wait for a single run to finish
TIMEOUT_SECONDS = 300 

# The directory where the .rs files are located (relative to this script)
BIN_DIR = "src/bin" # Change to "src/bin" if your files are inside src/bin

# Model configurations
MODELS = {
    'LI': {'file': 'main_li_esp.rs', 'bin': 'main_li_esp'},
    'HI': {'file': 'main_hi_esp.rs', 'bin': 'main_hi_esp'},
    # Fallback to main_ho_esp.rs if it exists, otherwise use main_ho.rs as stated in your prompt
    'HO': {'file': 'main_ho_esp.rs' if os.path.exists(os.path.join(BIN_DIR, 'main_ho_esp.rs')) else 'main_ho.rs', 'bin': 'main_ho'},
}

# --- REGEX PATTERNS ---
width_pattern = re.compile(r'const\s+WIDTH\s*:\s*usize\s*=\s*\d+\s*;')

# Time matching pattern (e.g., "[INFO ] Model creation time: 1536 µs")
time_pattern = re.compile(
    r'(Model creation time|Simulator creation time|Simulation time):\s*([\d\.]+)\s*([a-zA-Zµ]+)'
)

def update_source_file(model, filepath, width, depth):
    """Updates the WIDTH constant and generate_xx! macro in the source code."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # 1. Update const WIDTH
    content = width_pattern.sub(f'const WIDTH: usize = {width};', content)
    
    # 2. Update generate_xx!(width, depth) macro
    model_lower = model.lower()
    macro_pattern = re.compile(rf'generate_{model_lower}!\s*\(\s*\d+\s*,\s*\d+\s*\);')
    content = macro_pattern.sub(f'generate_{model_lower}!({width}, {depth});', content)

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

def to_seconds(value_str, unit):
    """Converts the extracted time into seconds."""
    val = float(value_str)
    unit = unit.lower()
    if unit in ['ns']: return val * 1e-9
    if unit in ['µs', 'us']: return val * 1e-6
    if unit in ['ms']: return val * 1e-3
    if unit in ['s']: return val
    return val # Fallback

def run_simulation(bin_name):
    """Executes cargo run, reads output, and captures the three metrics."""
    cmd = ["cargo", "run", "--quiet", "--release", "--package", "esp_devstone", "--bin", bin_name]
    
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        encoding='utf-8',
        errors='replace'
    )

    model_time = None
    runner_time = None
    sim_time = None

    start_time = time.time()

    # Read output line-by-line
    for line in iter(process.stdout.readline, ''):
        # Break if process closes prematurely
        if not line and process.poll() is not None:
            break

        sys.stdout.write(line) # Optional: echo output to console
        sys.stdout.flush()

        # Check for timeouts
        if time.time() - start_time > TIMEOUT_SECONDS:
            print(f"\n[!] Timeout waiting for {bin_name} to complete.")
            break

        match = time_pattern.search(line)
        if match:
            metric = match.group(1)
            val = match.group(2)
            unit = match.group(3)

            sec_val = to_seconds(val, unit)

            if metric == "Model creation time":
                model_time = sec_val
            elif metric == "Simulator creation time":
                runner_time = sec_val
            elif metric == "Simulation time":
                sim_time = sec_val

        # Stop executing once we have obtained all 3 times
        if model_time is not None and runner_time is not None and sim_time is not None:
            break

    # Once times are acquired, forcefully terminate the continuous serial monitor connection
    process.terminate()
    try:
        process.wait(timeout=3)
    except subprocess.TimeoutExpired:
        process.kill()

    return model_time, runner_time, sim_time

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    bin_full_dir = os.path.join(script_dir, BIN_DIR)

    if not os.path.exists(bin_full_dir):
        print(f"Error: The directory {bin_full_dir} does not exist.")
        sys.exit(1)

    for model, config in MODELS.items():
        csv_filename = f"results_{model}.csv"
        csv_filepath = os.path.join(script_dir, csv_filename)
        source_filepath = os.path.join(bin_full_dir, config['file'])

        # Create/check CSV
        file_exists = os.path.isfile(csv_filepath)
        with open(csv_filepath, 'a', newline='', encoding='utf-8') as csv_file:
            writer = csv.writer(csv_file, delimiter=';')
            if not file_exists:
                # Write header if new file
                writer.writerow([
                    'engine', 'iter', 'model', 'width', 'depth', 
                    'int_delay', 'ext_delay', 'model_time', 
                    'runner_time', 'sim_time', 'total_time'
                ])

            # Iterate over variables
            for width in WIDTHS:
                for depth in DEPTHS:
                    # 1. Mutate the source code parameters
                    print(f"\nUpdating {config['file']} -> WIDTH: {width}, DEPTH: {depth}")
                    update_source_file(model, source_filepath, width, depth)

                    # 2. Run simulation N iterations
                    for i in range(ITERATIONS):
                        print(f"  -> Running iteration {i}/{ITERATIONS - 1} for {model}...")
                        m_time, r_time, s_time = run_simulation(config['bin'])

                        if None in (m_time, r_time, s_time):
                            print(f"  [!] Failed to collect all metrics for iter {i}. Skipping row.")
                            continue
                        
                        # 3. Write data to CSV
                        total_time = m_time + r_time + s_time
                        writer.writerow([
                            "xdevs-rs-new-sequential", # engine
                            i,                         # iter
                            model,                     # model (LI, HI, HO)
                            width,                     # width
                            depth,                     # depth
                            0,                         # int_delay
                            0,                         # ext_delay
                            f"{m_time:.6g}",           # model_time
                            f"{r_time:.6g}",           # runner_time
                            f"{s_time:.6g}",           # sim_time
                            f"{total_time:.6g}"        # total_time
                        ])
                        # Flush file buffers to save progress instantly
                        csv_file.flush()

    print("\nAll experiments successfully completed!")

if __name__ == "__main__":
    main()