import sys
import json
import pathlib
import pandas as pd
import matplotlib.pyplot as plt

if __name__ == "__main__":
    arguments = sys.argv[1:]
    num_type = arguments[0]

    plt.rcParams['font.size'] = 18

    # -------------------------
    # 1️⃣ Load Criterion results
    # -------------------------

    criterion_dir = pathlib.Path("target/criterion")
    bench_results = []

    # Iterate all benchmarks, open only estimates.json
    for bench_folder in criterion_dir.glob("**/new"):
        estimates_file = bench_folder / "estimates.json"
        if num_type not in str(bench_folder):
            continue
        if estimates_file.exists():
            with open(estimates_file) as f:
                data = json.load(f)
                # use folder name as benchmark name
                name = bench_folder.parent.name  # <-- this is the crate-type-digits-... string
                mean_ns = data['mean']['point_estimate'] / 1000.0
                bench_results.append((name, mean_ns))

    # -------------------------
    # 2️⃣ Parse benchmark names
    # -------------------------

    def parse_bench_name(name):
        """
        Parse a benchmark name like 'cetane-u64-digits-1-to-2'
        Returns: crate, typ, min_digit, max_digit
        """
        parts = name.split("-")

        # find "digits" keyword
        try:
            digits_idx = parts.index("digits")
        except ValueError:
            raise ValueError(f"Cannot find 'digits' in benchmark name: {name}")

        crate = "-".join(parts[:digits_idx-1])  # if crate has hyphens, join them
        typ = parts[digits_idx-1]
        min_digit = int(parts[digits_idx + 1])
        # handle case "digits-5" vs "digits-1-to-5"
        if digits_idx + 3 < len(parts) and parts[digits_idx + 2] == "to":
            max_digit = int(parts[digits_idx + 3])
        else:
            max_digit = min_digit

        return crate, typ, min_digit, max_digit

    rows = []
    for name, mean_ns in bench_results:
        # expected format: <crate>-<type>-digits-<min>-to-<max>
        crate, typ, min_digit, max_digit = parse_bench_name(name)
        rows.append({
            "crate": crate,
            "type": typ,
            "min_digit": min_digit,
            "max_digit": max_digit,
            "mean_ns": mean_ns
        })

    df = pd.DataFrame(rows)

    df = df[df['type'] == num_type]

    # -------------------------
    # 3️⃣ Split data: exact digits vs 1..N
    # -------------------------
    exact_digits_df = df[df['min_digit'] == df['max_digit']]
    range_digits_df = df[df['min_digit'] == 1]  # 1..N distributions

    # -------------------------
    # 4️⃣ Plot exact digits (N to N)
    # -------------------------
    plt.figure(figsize=(10,6))
    for crate, group in exact_digits_df.groupby("crate"):
        group = group.sort_values("max_digit")  # <- sort by N
        plt.plot(group['max_digit'], group['mean_ns'], marker='o', label=crate, linewidth=2.5)

    plt.xlabel("Number of Digits")
    plt.ylabel("Parse Time (ns)")
    plt.title("%s Parser Performance: Exactly N Digits (Lower is Better)" % num_type)
    plt.legend(loc='upper left')
    plt.grid(True)
    # plt.tight_layout()
    plt.ylim(bottom=0) 
    plt.show()

    # -------------------------
    # 5️⃣ Plot 1..N distributions
    # -------------------------
    plt.figure(figsize=(10,6))
    for crate, group in range_digits_df.groupby("crate"):
        group = group.sort_values("max_digit")  # <- sort by N
        plt.plot(group['max_digit'], group['mean_ns'], marker='o', label=crate, linewidth=2.5)

    plt.xlabel("Number of Digits [1..N]")
    plt.ylabel("Parse Time (ns)")
    plt.title("%s Parser Performance: [1..N] Digit Range (Lower is Better)" % num_type)
    plt.legend(loc='upper left')
    plt.grid(True)
    # plt.tight_layout()
    plt.ylim(bottom=0) 
    plt.show()