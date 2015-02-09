extern crate "rustc-serialize" as rustc_serialize;
extern crate csv;

pub mod wca_data {
    use csv;
    use std::collections::HashMap;
    use std::collections::BTreeMap;
    use std::collections::HashSet;
    use std::collections::Bound::{Included, Unbounded};
    use rustc_serialize::Decodable;
    use rustc_serialize::Decoder;
    use std::old_path::posix::Path;

    pub type WcaId = String;
    pub type PuzzleId = String;

    #[derive(PartialEq, Clone, Copy)]
    pub enum Gender {
        Male,
        Female,
        Unknown,
    }

    #[derive(Copy)]
    pub enum ResultType {
        Single,
        Average,
    }

    impl Decodable for Gender {
        fn decode<D: Decoder>(d: &mut D) -> Result<Gender, D::Error> {
            match d.read_str() {
                Ok(s) => {
                    match s.as_slice() {
                        "m" => Ok(Gender::Male),
                        "f" => Ok(Gender::Female),
                        _   => Ok(Gender::Unknown),
                    }
                }
                Err(e) => Err(e),
            }
        }
    }

    #[derive(RustcDecodable)]
    struct Person {
        id: WcaId,
        subid: i32,
        name: String,
        country: String,
        gender: Gender,
    }

    pub struct Competitor {
        pub id: WcaId,
        pub name: String,
        pub country: String,
        pub gender: Gender,
        pub competition_count: u32,
    }

    #[derive(RustcDecodable)]
    struct Rank {
        person_id: WcaId,
        event_id: String,
        best: u32,
        world_rank: u32,
        continent_rank: u32,
        country_rank: u32,
    }

    pub struct Ranking {
        pub result: CompResult,
        pub competitor_id: WcaId,
    }

    // TODO add puzzle enum
    #[derive(RustcDecodable, RustcEncodable, Clone, Copy)]
    pub struct CompResult {
        pub time: u32,
    }

    #[derive(RustcDecodable, RustcEncodable, Clone)]
    pub struct Record {
        pub single: CompResult,
        pub average: Option<CompResult>,
    }

    #[derive(RustcEncodable)]
    pub struct RecordWithCompetitor {
        pub competitor_id: String,
        pub single: CompResult,
        pub average: Option<CompResult>,
    }

    pub struct WCA {
        pub persons: BTreeMap<WcaId, Competitor>,
        competitions: HashMap<WcaId, HashSet<String>>,
        records: HashMap<String, HashMap<String, Record>>,
        single_rankings: HashMap<PuzzleId, Vec<Ranking>>,
        average_rankings: HashMap<PuzzleId, Vec<Ranking>>,
        events: Vec<Event>,
    }

    #[derive(RustcDecodable, RustcEncodable)]
    pub struct Event {
        pub id: String,
        pub name: String,
    }

    impl WCA {
        fn insert_person(&mut self, person: Person) {
            let c = Competitor { id: person.id, name: person.name, gender: person.gender, country: person.country, competition_count: 0 };
            self.persons.insert(c.id.clone(), c);
        }

        fn visited_comp(&mut self, id: String, comp_id: String) {
            if self.competitions.contains_key(&id.clone()) {
            } else {
                self.competitions.insert(id.clone(), HashSet::new());
            }
            let set = self.competitions.get_mut(&id).unwrap();
            set.insert(comp_id);
        }

        fn update_competition_count_cache(&mut self) {
            for (id, competitor) in self.persons.iter_mut() {
                match self.competitions.get(id) {
                    Some(set) => { competitor.competition_count = set.len() as u32; },
                    None      => { },
                }
            }
        }

        fn add_single_record(&mut self, id: String, puzzle: String, time: u32) {
            if self.records.contains_key(&id.clone()) {
            } else {
                self.records.insert(id.clone(), HashMap::new());
            }
            let map = self.records.get_mut(&id).unwrap();
            map.insert(puzzle, Record{single: CompResult{time: time}, average: None});
        }

        fn add_single_ranking(&mut self, puzzle_id: String, best: u32, competitor_id: String) {
            if self.single_rankings.contains_key(&puzzle_id) {
            } else {
                self.single_rankings.insert(puzzle_id.clone(), vec![]);
            }
            let vec = self.single_rankings.get_mut(&puzzle_id).unwrap();
            vec.push(Ranking { result: CompResult {time: best}, competitor_id: competitor_id.clone()});
        }

        fn add_average_ranking(&mut self, puzzle_id: String, best: u32, competitor_id: String) {
            if self.average_rankings.contains_key(&puzzle_id) {
            } else {
                self.average_rankings.insert(puzzle_id.clone(), vec![]);
            }
            let vec = self.average_rankings.get_mut(&puzzle_id).unwrap();
            vec.push(Ranking { result: CompResult {time: best}, competitor_id: competitor_id.clone()});
        }

        fn add_average_record(&mut self, id: String, puzzle: String, time: u32) {
            // This assumes that
            // a) Adding single records have been executed first.
            // b) For every average record exists one single record.

            // FIXME this exists to hack around borrow checker
            let mut record;

            {
                let records = self.records.get(&id).unwrap();
                record = records.get(&puzzle).unwrap().clone();
            }

            self.records.get_mut(&id).unwrap().insert(puzzle, Record { single: record.clone().single, average: Some(CompResult{time: time}) });
        }

        pub fn number_of_comps(&self, id: &String) -> Option<usize> {
            self.competitions.get(id).map(|set| set.len())
        }

        pub fn find_competitor(&self, id: &String) -> Option<&Competitor> {
            self.persons.get(id)
        }

        pub fn find_competitors(&self, query: &String) -> Vec<&Competitor> {
            self.persons
                .range(Included(query), Unbounded)
                .take_while(|t| t.0.starts_with(query.as_slice()))
                .map(|t| t.1)
                .collect()
        }

        pub fn find_records(&self, competitor_id: &String) -> Option<&HashMap<String, Record>> {
            self.records.get(competitor_id)
        }

        pub fn find_events(&self) -> &Vec<Event> {
            &self.events
        }

        pub fn find_rankings(&self, puzzle_id: &String, result_type: ResultType) -> Option<&Vec<Ranking>> {
            match result_type {
                ResultType::Single  => self.single_rankings.get(puzzle_id),
                ResultType::Average => self.average_rankings.get(puzzle_id),
            }
        }

        pub fn find_rankings_for(&self, puzzle_id: &String, ids: Vec<String>) -> Vec<RecordWithCompetitor> {
            let mut result: Vec<RecordWithCompetitor> = ids.iter().filter_map(|id|
                self.find_records(id).map(|r|{
                    let record = r.get(puzzle_id);
                    match record {
                        Some(r) => Some(RecordWithCompetitor { single: r.clone().single, average: r.clone().average, competitor_id: id.to_string() }),
                        None => None,
                    }
                }).unwrap_or_else(|| None)
            ).collect();

            result.sort_by(|a, b|
                a.single.time.cmp(&b.single.time)
            );
            result
        }

        pub fn new(persons_path: &Path, results_path: &Path, records_single_path: &Path, records_average_path: &Path, events_path: &Path) -> Box<WCA> {
            let mut w = Box::new(WCA { persons: BTreeMap::new(), competitions: HashMap::new(), records: HashMap::new(), single_rankings: HashMap::new(), average_rankings: HashMap::new(), events: Vec::new() });
            load_persons(&mut *w, persons_path);
            load_competitions(&mut *w, results_path);
            load_single_records(&mut *w, records_single_path);
            load_average_records(&mut *w, records_average_path);
            load_events(&mut *w, events_path);
            w.update_competition_count_cache();
            w
        }
    }

    fn load_persons(w: &mut WCA, fp: &Path) {
        let mut rdr = csv::Reader::from_file(fp).has_headers(true).delimiter(b'\t');

        for record in rdr.decode() {
            let record: Person = record.unwrap();
            w.insert_person(record);
        }
    }

    fn load_competitions(w: &mut WCA, fp: &Path) {
        let mut rdr = csv::Reader::from_file(fp).has_headers(true).delimiter(b'\t');

        for record in rdr.decode() {
            let v: Vec<String> = record.unwrap();
            w.visited_comp(v[7].clone(), v[0].clone());
        }
    }

    fn load_single_records(w: &mut WCA, fp: &Path) {
        let mut rdr = csv::Reader::from_file(fp).has_headers(true).delimiter(b'\t');

        for record in rdr.decode() {
            let r: Rank = record.unwrap();
            w.add_single_record(r.person_id.clone(), r.event_id.clone(), r.best);
            w.add_single_ranking(r.event_id, r.best, r.person_id);
        }
        for (_, vec) in w.single_rankings.iter_mut() {
            let mut s = vec.as_mut_slice();
            s.sort_by(|a, b| a.result.time.cmp(&b.result.time));
        }
    }

    fn load_average_records(w: &mut WCA, fp: &Path) {
        let mut rdr = csv::Reader::from_file(fp).has_headers(true).delimiter(b'\t');

        for record in rdr.decode() {
            let r: Rank = record.unwrap();
            w.add_average_record(r.person_id.clone(), r.event_id.clone(), r.best);
            w.add_average_ranking(r.event_id, r.best, r.person_id);
        }
    }

    fn load_events(w: &mut WCA, fp: &Path) {
        let mut rdr = csv::Reader::from_file(fp).has_headers(true).delimiter(b'\t');

        for record in rdr.decode() {
            let e: Event = record.unwrap();
            w.events.push(e);
        }
    }

    pub fn build_from_files(persons_path: &Path,
                            results_path: &Path,
                            records_single_path: &Path,
                            records_average_path: &Path,
                            events_path: &Path) -> Box<WCA> {
        WCA::new(persons_path, results_path, records_single_path, records_average_path, events_path)
    }
}
