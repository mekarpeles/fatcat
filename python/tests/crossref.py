
import json
import pytest
from fatcat.crossref_importer import FatcatCrossrefImporter


@pytest.fixture(scope="function")
def crossref_importer():
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield FatcatCrossrefImporter("http://localhost:9411/v0", issn_file, 'tests/files/example_map.sqlite3')

def test_crossref_importer_batch(crossref_importer):
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        crossref_importer.process_batch(f)

def test_crossref_importer(crossref_importer):
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        crossref_importer.process_source(f)

def test_crossref_importer_create(crossref_importer):
    crossref_importer.create_containers = True
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        crossref_importer.process_source(f)

def test_crossref_dict_parse(crossref_importer):
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line
        raw = json.loads(f.read())
        (r, c) = crossref_importer.parse_crossref_dict(raw)
        extra = r.extra['crossref']
        assert r.title == "Renormalized perturbation theory by the moment method for degenerate states: Anharmonic oscillators"
        assert r.doi == "10.1002/(sici)1097-461x(1998)66:4<261::aid-qua1>3.0.co;2-t"
        assert r.publisher == "Wiley-Blackwell"
        print(extra)
        assert extra['container-title'] == ["International Journal of Quantum Chemistry"]
        assert r.release_type == "journal-article"
        assert r.release_status == "published"
        assert r.isbn13 == "978-3-16-148410-0"
        assert 'subtitle' not in extra
        assert 'archive' not in extra
        assert 'funder' not in extra
        assert len(r.contribs) == 5
        assert r.contribs[0].raw_name == "Marcelo D. Radicioni"
        assert r.contribs[0].index == 0
        assert r.contribs[1].extra['affiliations'] == ["Some University"]
        assert r.contribs[1].role == "author"
        assert r.contribs[3].role == "editor"
        assert r.contribs[3].index is None
        assert r.contribs[4].role == "translator"
        assert r.contribs[4].index is None
        assert len(r.refs) == 25
        assert r.refs[0].key == "BIB1"
        assert r.refs[0].year == 1972
        assert r.refs[0].locator == "1734"
        assert r.refs[0].container_name == "J. Chem. Phys."
        assert r.refs[0].extra['crossref'] == {"volume": "57", "author": "Swenson", "doi": "10.1063/1.1678462"}
        assert r.refs[3].container_name == "Large Order Perturbation Theory and Summation Methods in Quantum Mechanics, Lecture Notes in Chemistry"
