from tarfile import TarInfo

import click
import puremagic as magic

import tempfile
import json
import tarfile
import gzip
import shutil
from pathlib import Path

from puremagic import PureError

from heuristics import get_maindoc, EXCLUDED_SAMPLES


def tar_filter(member: TarInfo, path: str) -> TarInfo | None:
    member.name = member.name.replace(':', '_colon_').replace('\\', '_backslash_').replace('\n', '_newline_').replace(
        '\r', '_carriage_return_').replace('\t', '_tab_')
    return member


class TestEnv(object):
    def __init__(self, sample: Path):
        self.tmpdir = Path(tempfile.mkdtemp('ttrac'))
        assert magic.from_extension(magic.ext_from_filename(sample)) == 'application/gzip'

        submission_data_path = self.tmpdir / sample.stem

        with gzip.open(sample) as gz:
            with open(submission_data_path, "wb") as f:
                shutil.copyfileobj(gz, f)

        try:
            ext = magic.from_extension(magic.ext_from_filename(submission_data_path))
        except PureError:
            ext = None
        if ext == "application/x-tar":
            with tarfile.open(submission_data_path, 'r') as tar:
                tar.extractall(path=self.tmpdir, filter=tar_filter)
            submission_data_path.unlink()

    def __enter__(self):
        return self.tmpdir

    def __exit__(self, exc, value, tb):
        shutil.rmtree(self.tmpdir)


def prepare(sample: Path) -> str | None:
    if sample.stat().st_size < 100:
        # submission was withdrawn
        return None
    if sample.stem in EXCLUDED_SAMPLES:
        return None

    with TestEnv(sample) as d:
        maindoc = get_maindoc(d, sample)
        if maindoc:
            return maindoc.name
    return None


@click.command()
@click.argument('corpus', type=click.Path(exists=True))
def prepare_dataset(corpus):
    output = {}
    output_path = corpus + ".json"
    assert corpus[-1] != "/"

    for sample in Path(corpus).iterdir():
        res = prepare(sample)
        if res:
            print(sample.stem, res)
            output[sample.stem] = res

    with open(output_path, "w") as f:
        json.dump(output, f)


if __name__ == '__main__':
    prepare_dataset()
