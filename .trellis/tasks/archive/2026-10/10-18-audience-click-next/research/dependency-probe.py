"""Read public upstream release metadata; only write research evidence."""
import concurrent.futures
import json
from pathlib import Path
from urllib.request import urlopen

ROOT = Path(__file__).parent
PACKAGES = ["zbus", "zvariant", "zbus_names", "zvariant_derive", "ashpd", "oo7", "notify-rust", "accesskit_unix", "accesskit_atspi_common", "atspi", "atspi-common", "atspi-proxies", "zbus-lockstep", "zbus_xml", "zgvariant"]


def read(name):
    with urlopen(f"https://crates.io/api/v1/crates/{name}", timeout=30) as response:
        info = json.load(response)
    stable = next(v for v in info["versions"] if not v["yanked"] and "-" not in v["num"])
    versions = [stable]
    if info["versions"][0]["num"] != stable["num"]:
        versions.append(info["versions"][0])
    result = []
    for version in versions:
        number = version["num"]
        url = f"https://crates.io/api/v1/crates/{name}/{number}/dependencies"
        with urlopen(url, timeout=30) as response:
            deps = json.load(response)["dependencies"]
        relevant = [d for d in deps if d["kind"] != "dev" and (d["crate_id"] in PACKAGES or d["crate_id"] in ["accesskit", "accesskit_consumer", "zbus_macros", "zruntime"])]
        result.append({"name": name, "version": number, "rust_version": version.get("rust_version"), "created_at": version["created_at"], "source": url, "dependencies": relevant})
    return result


records = list(concurrent.futures.ThreadPoolExecutor(max_workers=5).map(read, PACKAGES))
flat = [r for rs in records for r in rs]
(ROOT / "dependency-upstream-metadata.json").write_text(json.dumps(flat, indent=2) + "\n", encoding="utf-8")
for record in flat:
    print(record["name"], record["version"], "MSRV", record["rust_version"], "; ".join(f"{d['crate_id']} {d['req']}" for d in record["dependencies"]))
