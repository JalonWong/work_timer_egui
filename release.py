import platform
import subprocess
import sys
from glob import glob

PYTHON = sys.executable
SYSTEM = platform.system().lower()
EXT = ".exe" if SYSTEM == "windows" else ""
if SYSTEM == "darwin":
    SYSTEM = "osx"
ARCH = platform.machine().lower()


if __name__ == "__main__":
    print("Machine:", ARCH, flush=True)
    subprocess.run([PYTHON, "--version"])
    subprocess.run("cargo --version".split(), check=True)

    subprocess.run("cargo install cargo-bundle".split(), check=True)
    subprocess.run("cargo build --release".split(), check=True)

    if SYSTEM == "windows":
        import zipfile

        arch = "arm64" if ARCH == "arm64" else "amd64"
        with zipfile.ZipFile(f"work-timer-{SYSTEM}-{arch}.zip", "w", zipfile.ZIP_DEFLATED) as zip_f:
            zip_f.write(f"target/release/work_timer{EXT}", f"work_timer{EXT}")
            files = glob("assets/**")
            for file in files:
                zip_f.write(file)
    elif SYSTEM == "osx":
        subprocess.run("cargo bundle --release".split(), check=True)

        import zipfile

        arch = "arm64" if ARCH == "arm64" else "amd64"
        with zipfile.ZipFile(f"work-timer-{SYSTEM}-{arch}.zip", "w", zipfile.ZIP_DEFLATED) as zip_f:
            files = glob(
                "target/release/bundle/osx/WorkTimer.app/**", recursive=True, include_hidden=True
            )
            for file in files:
                zip_f.write(file, file.replace("target/release/bundle/osx/", ""))
    elif SYSTEM == "linux":
        import tarfile

        with tarfile.open("work-timer-linux-amd64.tar.gz", "w:gz") as tar:
            tar.add("target/release/work_timer", "work_timer")
            files = glob("assets/**")
            for file in files:
                tar.add(file)
