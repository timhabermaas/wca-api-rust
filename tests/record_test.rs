extern crate "wca-data" as w;

use w::wca_data;
use w::wca_data::WCA;

fn setup_data() -> Box<WCA> {
    wca_data::build_from_files(&Path::new("./tests/fixtures/persons.tsv"), &Path::new("./tests/fixtures/results.tsv"), &Path::new("./tests/fixtures/ranks-single.tsv"), &Path::new("./tests/fixtures/ranks-average.tsv"), &Path::new("./tests/fixtures/events.tsv"))
}

#[test]
fn single_records() {
    let w = setup_data();
    let record = w.find_records(&"2005AKKE01".to_string()).unwrap();
    let three_by_three = record.get(&"333".to_string()).unwrap();
    let four_by_four   = record.get(&"444".to_string()).unwrap();

    assert_eq!(three_by_three.single.time, 708u);
    assert_eq!(four_by_four.single.time, 2999);
}

#[test]
fn average_records() {
    let w = setup_data();
    let record = w.find_records(&"2005AKKE01".to_string()).unwrap();

    let three_by_three = record.get(&"333".to_string()).unwrap();
    let blindfolded_44 = record.get(&"444bf".to_string()).unwrap();

    assert_eq!(three_by_three.average.unwrap().time, 931u);
    assert_eq!(blindfolded_44.average.is_none(), true);
}

#[test]
fn single_rankings() {
    let w = setup_data();
    let ranks = w.find_rankings(&"333".to_string(), wca_data::Single).unwrap();
    assert_eq!(ranks.len(), 4);
    assert_eq!(ranks.get(0).unwrap().result.time, 708u);
    assert_eq!(ranks.get(0).unwrap().competitor_id, "2005AKKE01".to_string());
    assert_eq!(ranks.get(1).unwrap().result.time, 871);
    assert_eq!(ranks.get(1).unwrap().competitor_id, "2003BRUC01".to_string());
    assert_eq!(ranks.get(2).unwrap().result.time, 1065);
    assert_eq!(ranks.get(2).unwrap().competitor_id, "2007WEIN01".to_string());
    assert_eq!(ranks.get(3).unwrap().result.time, 4647);
    assert_eq!(ranks.get(3).unwrap().competitor_id, "2011RAHM01".to_string());

    let ranks = w.find_rankings(&"444bf".to_string(), wca_data::Single).unwrap();
    assert_eq!(ranks.len(), 1);
    assert_eq!(ranks.get(0).unwrap().competitor_id, "2005AKKE01".to_string());
}

#[test]
fn average_rankings() {
    let w = setup_data();
    let ranks = w.find_rankings(&"333".to_string(), wca_data::Average).unwrap();
    assert_eq!(ranks.len(), 2);
    assert_eq!(ranks.get(0).unwrap().result.time, 931u);
    assert_eq!(ranks.get(0).unwrap().competitor_id, "2005AKKE01".to_string());
    assert_eq!(ranks.get(1).unwrap().result.time, 1262u);
    assert_eq!(ranks.get(1).unwrap().competitor_id, "2003BRUC01".to_string());
}
