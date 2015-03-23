extern crate "wca-data" as w;

use w::wca_data;
use w::wca_data::WCA;
use std::path::Path;

fn setup_data() -> Box<WCA> {
    wca_data::build_from_files(Path::new("./tests/fixtures/persons.tsv"), Path::new("./tests/fixtures/results.tsv"), Path::new("./tests/fixtures/ranks-single.tsv"), Path::new("./tests/fixtures/ranks-average.tsv"), Path::new("./tests/fixtures/events.tsv"))
}

#[test]
fn events() {
    let w = setup_data();
    let events = w.find_events();
    assert_eq!(events.len(), 35);

    assert_eq!(events.get(0).unwrap().name, "Rubik's Cube".to_string());
    assert_eq!(events.get(0).unwrap().id, "333".to_string());

    assert_eq!(events.get(4).unwrap().name, "Rubik's Cube: Blindfolded".to_string());
    assert_eq!(events.get(4).unwrap().id, "333bf".to_string());
}
