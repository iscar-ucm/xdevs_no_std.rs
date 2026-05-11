import subprocess #ejecuta comandos externos desde python (para ejecutar: cargo run --example main_li --release)
import re #modifica texto en Rust
import csv #para almacenar en un archivo .csv los datos
#import shutil
import time #para medir tiempo
from pathlib import Path
from datetime import datetime
from itertools import product

#Rutas
BASE_DIR   = Path(r"C:/Users/Cristina/Documents/Cris/Teleco_UPM/Cuarto/TFG/xdevs_no_std.rs")
EXAMPLES   = BASE_DIR / "xdevs_devstone" / "examples"
MAIN_FILE  = EXAMPLES / "main_li.rs"
RESULTS_DIR = BASE_DIR / "benchmark_results"
COPIES_DIR  = RESULTS_DIR / "main_copies"

#Parámetros 
WIDTHS = range(1, 100)   
DEPTHS = range(1, 100)  

# Número de repeticiones por configuración (para promediar tiempos)
REPETITIONS = 1

# Tiempo límite por ejecución en segundos (None = sin límite)
TIMEOUT = None

# ── Regex para capturar tiempos impresos por el main ─────────────────────────
# Ajusta si cambias los println! de Rust
RE_MODEL  = re.compile(r"Model creation time:\s*([\d.]+(?:µs|ms|s|ns))")
RE_SIM    = re.compile(r"Simulation time:\s*([\d.]+(?:µs|ms|s|ns))")
RE_SIMCR  = re.compile(r"Simulator creation time:\s*([\d.]+(?:µs|ms|s|ns))")


def to_microseconds(duration_str: str) -> float:
    """Convierte una cadena de duración de Rust (e.g. '1.23ms') a microsegundos."""
    duration_str = duration_str.strip()
    if duration_str.endswith("ns"):
        return float(duration_str[:-2]) / 1_000
    elif duration_str.endswith("µs"):
        return float(duration_str[:-2])
    elif duration_str.endswith("ms"):
        return float(duration_str[:-2]) * 1_000
    elif duration_str.endswith("s"):
        return float(duration_str[:-1]) * 1_000_000
    return float("nan")


def patch_main(width: int, depth: int) -> str:
    """Lee main_li.rs, sustituye WIDTH y DEPTH y devuelve el contenido modificado."""
    original = MAIN_FILE.read_text(encoding="utf-8")

    # Sustituye las líneas const WIDTH/DEPTH dentro de main() y test_li()
    patched = re.sub(
        r"(const WIDTH\s*:\s*usize\s*=\s*)\d+\s*;",
        rf"\g<1>{width};",
        original,
    )
    patched = re.sub(
        r"(const DEPTH\s*:\s*usize\s*=\s*)\d+\s*;",
        rf"\g<1>{depth};",
        patched,
    )
    # W = WIDTH - 1
    patched = re.sub(
        r"(const W\s*:\s*usize\s*=\s*WIDTH\s*-\s*)\d+\s*;",
        r"\g<1>1;",
        patched,
    )
    return patched


def uncomment_timings(content: str) -> str:
    """Descomenta los println! de tiempos si están comentados."""
    lines = []
    for line in content.splitlines():
        stripped = line.lstrip()
        # Descomenta // println!("Model creation time: ...") etc.
        if stripped.startswith("// ") and "println!" in stripped and "time" in stripped.lower():
            lines.append(line.replace("// ", "", 1))
        else:
            lines.append(line)
    return "\n".join(lines)


def run_once(width: int, depth: int) -> dict:
    """Ejecuta cargo run para una configuración y devuelve los tiempos capturados."""
    result = {
        "width": width,
        "depth": depth,
        "model_creation_us": float("nan"),
        "simulator_creation_us": float("nan"),
        "simulation_us": float("nan"),
        "returncode": -1,
        "error": "",
    }

    t_wall_start = time.perf_counter()
    try:
        proc = subprocess.run(
            ["cargo", "run", "--example", "main_li", "--release"],
            cwd=BASE_DIR / "xdevs_devstone",
            capture_output=True,
            text=True,
            timeout=TIMEOUT,
        )
        result["returncode"] = proc.returncode
        output = proc.stdout + proc.stderr

        m = RE_MODEL.search(output)
        if m:
            result["model_creation_us"] = to_microseconds(m.group(1))

        m = RE_SIMCR.search(output)
        if m:
            result["simulator_creation_us"] = to_microseconds(m.group(1))

        m = RE_SIM.search(output)
        if m:
            result["simulation_us"] = to_microseconds(m.group(1))

        if proc.returncode != 0:
            result["error"] = (proc.stderr or proc.stdout)[-500:]

    except subprocess.TimeoutExpired:
        result["error"] = f"Timeout ({TIMEOUT}s)"
    except Exception as exc:
        result["error"] = str(exc)

    result["wall_us"] = (time.perf_counter() - t_wall_start) * 1_000_000
    return result


def save_copy(width: int, depth: int, content: str):
    """Guarda una copia del main_li.rs modificado."""
    dest = COPIES_DIR / f"main_li_W{width}_D{depth}.rs"
    dest.write_text(content, encoding="utf-8")


def main():
    # Prepara directorios de salida
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    COPIES_DIR.mkdir(parents=True, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    csv_path = RESULTS_DIR / f"benchmark_{timestamp}.csv"

    fieldnames = [
        "width", "depth", "repetition",
        "model_creation_us", "simulator_creation_us", "simulation_us",
        "wall_us", "returncode", "error",
    ]

    original_content = MAIN_FILE.read_text(encoding="utf-8")

    try:
        with open(csv_path, "w", newline="", encoding="utf-8") as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()

            combos = list(product(WIDTHS, DEPTHS))
            total = len(combos) * REPETITIONS
            done = 0

            for width, depth in combos:
                patched = patch_main(width, depth)
                patched = uncomment_timings(patched)

                # Guarda copia del archivo modificado
                save_copy(width, depth, patched)

                # Escribe el archivo modificado
                MAIN_FILE.write_text(patched, encoding="utf-8")
                print(f"\n{'='*60}")
                print(f"  WIDTH={width}, DEPTH={depth}")
                print(f"{'='*60}")

                for rep in range(1, REPETITIONS + 1):
                    done += 1
                    print(f"  Repetición {rep}/{REPETITIONS}  (total: {done}/{total})", end=" ... ")
                    row = run_once(width, depth)
                    row["repetition"] = rep

                    if row["returncode"] == 0:
                        print(
                            f"OK  sim={row['simulation_us']:.1f} µs  "
                            f"wall={row['wall_us']/1e6:.2f} s"
                        )
                    else:
                        print(f"ERROR: {row['error'][:80]}")

                    writer.writerow(row)
                    csvfile.flush()

    finally:
        # Restaura el main original pase lo que pase
        MAIN_FILE.write_text(original_content, encoding="utf-8")
        print(f"\nmain_li.rs restaurado a su estado original.")

    print(f"\nResultados guardados en: {csv_path}")
    print(f"Copias de main_li.rs en:  {COPIES_DIR}")

    # ── Resumen estadístico básico ────────────────────────────────────────────
    try:
        import statistics
        rows_by_config: dict = {}
        with open(csv_path, newline="", encoding="utf-8") as f:
            for row in csv.DictReader(f):
                key = (int(row["width"]), int(row["depth"]))
                rows_by_config.setdefault(key, []).append(row)

        summary_path = RESULTS_DIR / f"summary_{timestamp}.csv"
        with open(summary_path, "w", newline="", encoding="utf-8") as f:
            writer = csv.writer(f)
            writer.writerow([
                "width", "depth",
                "sim_mean_us", "sim_stdev_us",
                "wall_mean_s", "ok_runs",
            ])
            for (w, d), rows in sorted(rows_by_config.items()):
                ok = [r for r in rows if r["returncode"] == "0"]
                sims = [float(r["simulation_us"]) for r in ok if r["simulation_us"] != "nan"]
                walls = [float(r["wall_us"]) for r in ok]
                if sims:
                    writer.writerow([
                        w, d,
                        f"{statistics.mean(sims):.1f}",
                        f"{statistics.stdev(sims):.1f}" if len(sims) > 1 else "0",
                        f"{statistics.mean(walls)/1e6:.3f}",
                        len(ok),
                    ])
        print(f"Resumen estadístico en:   {summary_path}")
    except Exception as exc:
        print(f"No se pudo generar el resumen: {exc}")


if __name__ == "__main__":
    main()