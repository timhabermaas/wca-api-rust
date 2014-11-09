extern crate "wca-data" as w;

use w::wca_data;
use w::wca_data::WCA;

fn setup_data() -> Box<WCA> {
    let mut w = wca_data::build_from_files(&Path::new("./tests/fixtures/persons.tsv"), &Path::new("./tests/fixtures/results.tsv"), &Path::new("./tests/fixtures/ranks-single.tsv"), &Path::new("./tests/fixtures/ranks-average.tsv"), &Path::new("./tests/fixtures/events.tsv"));
    w
}


#[test]
fn competitor_gender_female() {
    let mut w = setup_data();
    let c = w.find_competitor(&"1982FRID01".to_string()).unwrap();
    assert!(c.gender == wca_data::Female);
}

#[test]
fn competitor_gender_unknown() {
    let mut w = setup_data();
    let c = w.find_competitor(&"2014RODR25".to_string()).unwrap();
    assert!(c.gender == wca_data::Unknown);
}

#[test]
fn competition_count() {
    let mut w = setup_data();
    let count = w.number_of_comps(&"1982RAZO01".to_string()).unwrap();
    assert_eq!(count, 3u);
}

#[test]
fn test_find_competitors_one() {
    let mut w = setup_data();
    let competitors = w.find_competitors(&"1982FRID01".to_string());
    assert_eq!(competitors.len(), 1);
}

#[test]
fn test_find_competitors_several() {
    let mut w = setup_data();
    let competitors = w.find_competitors(&"1982L".to_string());
    assert_eq!(competitors.len(), 2);

    assert_eq!(competitors.iter().find(|c| c.name == "Luc Van Laethem".to_string()).is_some(), true);
    assert_eq!(competitors.iter().find(|c| c.name == "Zoltán Lábas".to_string()).is_some(), true);
}
