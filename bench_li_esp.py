#!/usr/bin/env python3
"""
Benchmark automático para xdevs_devstone en ESP32-C6.

Para cada valor de WIDTH en [2, 4, 6, ..., 30] y 10 iteraciones:
  - Reescribe src/main.rs con ese WIDTH
  - Compila y flashea con espflash
  - Captura la salida serie y extrae los tiempos
  - Guarda resultados en benchmark_results.csv

Requisitos:
  pip install pyserial
  cargo install espflash          (o usa espflash del PATH)

Uso:
  python run_benchmark.py --port /dev/ttyUSB0 [--chip esp32c6]
"""

import argparse
import csv
import re
import subprocess
import sys
import time
from pathlib import Path

import serial

# ---------------------------------------------------------------------------
# Plantilla de src/main.rs  — se reescribe en cada iteración
# ---------------------------------------------------------------------------
MAIN_RS_TEMPLATE = """\
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{{Duration, Instant}};
use esp_println as _;

use xdevs_devstone::common::*;
use xdevs_devstone::li::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {{
    loop {{}}
}}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {{
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 65536);

    const WIDTH: usize = {width};
    const W: usize = WIDTH - 1;

    let start = Instant::now();
    xdevs_devstone_macros::generate_li!({width}, {width});
    let generator = Generator::new(5);
    let modelo_final: ModeloFinal<W> = ModeloFinal::build(generator, model_li);
    let model_creation: Duration = start.elapsed();
    info!("MODEL_TIME_US:{{}}", model_creation.as_micros());

    let start = Instant::now();
    let mut simulator = xdevs::simulator::Simulator::new(modelo_final);
    let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
    let simulator_creation: Duration = start.elapsed();
    info!("SIM_CREATION_US:{{}}", simulator_creation.as_micros());

    let start = Instant::now();
    simulator.simulate_vt(&config);
    let simulation: Duration = start.elapsed();
    info!("SIM_TIME_US:{{}}", simulation.as_micros());

    info!("BENCHMARK_DONE");

    loop {{
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {{}}
    }}
}}
"""

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def patch_main_rs(src_path: Path, width: int):
    src_path.write_text(MAIN_RS_TEMPLATE.format(width=width), encoding="utf-8")
    print(f"  [patch] WIDTH={width} escrito en {src_path}")


def cargo_build(project_dir: Path, chip: str) -> bool:
    cmd = [
        "cargo", "build", "--release",
        "--target", f"{chip}-unknown-none-elf",
    ]
    print(f"  [build] {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=project_dir, capture_output=True, text=True)
    if result.returncode != 0:
        print("  [ERROR] cargo build falló:")
        print(result.stderr[-2000:])
        return False
    return True


def flash_and_capture(
    project_dir: Path,
    port: str,
    chip: str,
    baud_flash: int = 460800,
    baud_monitor: int = 115200,
    timeout_s: int = 60,
) -> dict | None:
    """Flashea el binario y lee la salida serie hasta encontrar BENCHMARK_DONE."""

    elf_glob = list(
        (project_dir / "target" / f"{chip}-unknown-none-elf" / "release").glob("*.elf")
    )
    # Descarta ficheros con '.' en el nombre (artefactos de cargo)
    elf_files = [f for f in elf_glob if "." not in f.stem]
    if not elf_files:
        # Fallback: cualquier ELF sin extensión
        elf_files = [f for f in elf_glob if not f.suffix]
    if not elf_files:
        print("  [ERROR] No se encontró el ELF compilado.")
        return None
    elf_path = elf_files[0]

    cmd_flash = [
        "espflash", "flash",
        "--chip", chip,
        "--port", port,
        "--baud", str(baud_flash),
        "--no-stub",
        str(elf_path),
    ]
    print(f"  [flash] {' '.join(cmd_flash)}")
    result = subprocess.run(cmd_flash, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        print("  [ERROR] espflash falló:")
        print(result.stderr[-1000:])
        return None

    # Pequeña pausa para que la placa reinicie
    time.sleep(1.5)

    # Leer salida serie
    print(f"  [monitor] Leyendo {port} a {baud_monitor} baud (timeout={timeout_s}s)…")
    times = {}
    deadline = time.time() + timeout_s
    try:
        with serial.Serial(port, baud_monitor, timeout=1) as ser:
            while time.time() < deadline:
                raw = ser.readline()
                if not raw:
                    continue
                line = raw.decode("utf-8", errors="replace").strip()
                if line:
                    print(f"    >> {line}")
                if m := re.search(r"MODEL_TIME_US:(\d+)", line):
                    times["model_us"] = int(m.group(1))
                if m := re.search(r"SIM_CREATION_US:(\d+)", line):
                    times["sim_creation_us"] = int(m.group(1))
                if m := re.search(r"SIM_TIME_US:(\d+)", line):
                    times["sim_time_us"] = int(m.group(1))
                if "BENCHMARK_DONE" in line:
                    break
            else:
                print("  [WARN] Timeout esperando BENCHMARK_DONE")
    except serial.SerialException as exc:
        print(f"  [ERROR] Serie: {exc}")
        return None

    if len(times) < 3:
        print(f"  [WARN] Datos incompletos: {times}")
        return None

    return times


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="ESP32-C6 devstone benchmark runner")
    parser.add_argument("--port", required=True, help="Puerto serie, p.ej. /dev/ttyUSB0 o COM3")
    parser.add_argument("--chip", default="esp32c6", help="Chip target (default: esp32c6)")
    parser.add_argument("--project", default="C:/Users/Cristina/Documents/Cris/Teleco_UPM/Cuarto/TFG/xdevs_no_std.rs/esp_devstone/src/bin/main_li_es", help="Directorio raíz del proyecto Rust")
    parser.add_argument("--src", default="src/main.rs", help="Ruta relativa a main.rs")
    parser.add_argument("--output", default="benchmark_results.csv", help="Fichero CSV de salida")
    parser.add_argument("--iterations", type=int, default=10, help="Iteraciones por WIDTH")
    parser.add_argument(
        "--widths",
        nargs="+",
        type=int,
        default=list(range(2, 32, 2)),
        help="Valores de WIDTH a probar (default: 2 4 6 … 30)",
    )
    args = parser.parse_args()

    project_dir = Path(args.project).resolve()
    src_path = project_dir / args.src
    output_path = Path(args.output)

    if not src_path.exists():
        print(f"ERROR: no encuentro {src_path}")
        sys.exit(1)

    # Guardar main.rs original para restaurarlo al final
    original_src = src_path.read_text(encoding="utf-8")

    fieldnames = [
        "width", "iteration",
        "model_creation_us", "sim_creation_us", "sim_time_us",
        "model_creation_ms", "sim_creation_ms", "sim_time_ms",
    ]

    with output_path.open("w", newline="", encoding="utf-8") as csv_file:
        writer = csv.DictWriter(csv_file, fieldnames=fieldnames)
        writer.writeheader()

        total = len(args.widths) * args.iterations
        done = 0

        try:
            for width in args.widths:
                print(f"\n{'='*60}")
                print(f"WIDTH = {width}  ({args.iterations} iteraciones)")
                print(f"{'='*60}")

                patch_main_rs(src_path, width)

                if not cargo_build(project_dir, args.chip):
                    print(f"  [SKIP] WIDTH={width} descartado por error de compilación.")
                    continue

                for it in range(1, args.iterations + 1):
                    done += 1
                    print(f"\n  --- Iteración {it}/{args.iterations}  (global {done}/{total}) ---")
                    times = flash_and_capture(project_dir, args.port, args.chip)

                    if times is None:
                        print(f"  [SKIP] Iteración {it} de WIDTH={width} sin datos.")
                        continue

                    row = {
                        "width": width,
                        "iteration": it,
                        "model_creation_us": times["model_us"],
                        "sim_creation_us": times["sim_creation_us"],
                        "sim_time_us": times["sim_time_us"],
                        "model_creation_ms": round(times["model_us"] / 1000, 3),
                        "sim_creation_ms": round(times["sim_creation_us"] / 1000, 3),
                        "sim_time_ms": round(times["sim_time_us"] / 1000, 3),
                    }
                    writer.writerow(row)
                    csv_file.flush()
                    print(
                        f"  [OK] model={row['model_creation_ms']} ms  "
                        f"sim_create={row['sim_creation_ms']} ms  "
                        f"sim={row['sim_time_ms']} ms"
                    )

        finally:
            # Restaurar main.rs original pase lo que pase
            src_path.write_text(original_src, encoding="utf-8")
            print(f"\n[INFO] main.rs restaurado a su estado original.")

    print(f"\n[DONE] Resultados guardados en {output_path.resolve()}")


if __name__ == "__main__":
    main()