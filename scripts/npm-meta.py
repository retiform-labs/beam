# Re-apply npm metadata that wasm-pack cannot generate.
#
# wasm-pack derives pkg/package.json from Cargo.toml alone: it takes the crate
# name (beamdb, unscoped) and emits no author field at all. So every build
# wipes the @retiform scope, the author and the collaborators -- and a publish
# straight after a build would either fail (the unscoped name is owned by
# someone else) or ship a package with an empty author.
#
# Cargo cannot carry a scoped name (@scoped/x is not a valid crate name), so
# the patch has to run after the build. Both build sites in the justfile call
# this script; keep the values here as the single source of truth.
import json
import pathlib
import sys

PKG = pathlib.Path(__file__).resolve().parent.parent / 'pkg' / 'package.json'

METADATA = {
    'name': '@retiform/beamdb',
    'author': 'Guan <guan@retiform.dev>',
    'collaborators': [
        'Martti Malmi <martti@mmalmi.org>',
        'Guan <guan@retiform.dev>',
    ],
    'repository': {'type': 'git', 'url': 'https://github.com/retiform-labs/beam'},
    'homepage': 'https://github.com/retiform-labs/beam',
    'license': 'MIT',
}


def main() -> int:
    if not PKG.is_file():
        print('npm-meta: pgg/package.json missing -- run wasm-pack build first',
              file=sys.stderr)
        return 1
    data = json.loads(PKG.read_text())
    data.update(METADATA)
    PKG.write_text(json.dumps(data, indent=2) + '\n')
    print('npm-meta: %s@%s author=%s' % (data['name'], data['version'], data['author']))
    return 0


if __name__ == '__main__':
    sys.exit(main())
