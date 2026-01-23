import os
import shutil
import subprocess
from argparse import ArgumentParser

parser = ArgumentParser()
parser.add_argument("-v", "--version")
parser.add_argument("-p", "--path")
parser.add_argument("-z", "--zip", type=bool, default=False)
parser.add_argument("-f", "--force", type=bool, default=True)


def generate():
    args = parser.parse_args()

    version = args.version
    if version is None:
        import toml  # pylint: disable=import-outside-toplevel
        with open("./Cargo.toml", "r", encoding='utf-8') as config:
            config_data = toml.load(config)
            version = config_data["package"]["version"]

    print(f"Client version: {version}")

    path = args.path
    if path is None:
        path = f'{os.path.expanduser("~")}/Dropbox/Brilliance/windows-build-{version}'

    if os.path.exists(path):
        if args.force:
            shutil.rmtree(path)
        else:
            raise FileExistsError(f'Path "{path}" already exists')

    os.makedirs(path, exist_ok=True)

    print('Building exe')
    result = subprocess.run(
        [
            "godot",
            "--export-release",
            "windows_desktop",
            f"{path}/Brilliance.exe",
        ],
        cwd=f"{os.path.expanduser("~")}/Projects/In-Its-Brilliance/brilliance-godot/",
        check=True,
    )

    if result.returncode != 0:
        raise RuntimeError(f"Godot export failed with code {result.returncode}")

    if args.zip:
        print('Creating zip')
        shutil.make_archive(path, 'zip', path)

    print('Complited')


if __name__ == '__main__':
    generate()
