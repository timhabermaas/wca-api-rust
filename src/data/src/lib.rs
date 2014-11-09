extern crate serialize;

pub mod wca_data {
    extern crate csv;

    use std::collections::HashMap;
    use std::collections::TreeMap;
    use std::collections::HashSet;
    use serialize::Decoder;
    use serialize::Decodable;
    use std::path::Path;
    pub type WcaId = String;
    pub type PuzzleId = String;

    #[deriving(PartialEq)]
    pub enum Gender {
        Male,
        Female,
        Unknown,
    }

    pub enum ResultType {
        Single,
        Average,
    }

    impl<E, D: Decoder<E>> Decodable<D, E> for Gender {
        fn decode(d: &mut D) -> Result<Gender, E> {
            match d.read_str() {
                Ok(s) => {
                    match s.as_slice() {
                        "m" => Ok(Male),
                        "f" => Ok(Female),
                        _   => Ok(Unknown),
                    }
                }
                Err(e) => Err(e),
            }
        }
    }

    #[deriving(Decodable)]
    pub struct Person {
        pub id: WcaId,
        subid: int,
        pub name: String,
        pub country_id: String,
        pub gender: Gender,
    }

    #[deriving(Decodable)]
    struct Rank {
        person_id: WcaId,
        event_id: String,
        best: uint,
        world_rank: uint,
        continent_rank: uint,
        country_rank: uint,
    }

    pub struct Ranking {
        pub result: CompResult,
        pub competitor_id: WcaId,
    }

    // TODO add puzzle enum
    #[deriving(Decodable, Encodable)]
    pub struct CompResult {
        pub time: uint,
    }

    #[deriving(Decodable, Encodable)]
    pub struct Record {
        pub single: CompResult,
        pub average: Option<CompResult>,
    }

    pub struct WCA {
        pub persons: TreeMap<WcaId, Person>,
        competitions: HashMap<WcaId, HashSet<String>>,
        records: HashMap<String, HashMap<String, Record>>,
        single_rankings: HashMap<PuzzleId, Vec<Ranking>>,
        average_rankings: HashMap<PuzzleId, Vec<Ranking>>,
        events: Vec<Event>,
    }

    #[deriving(Decodable, Encodable)]
    pub struct Event {
        pub id: String,
        pub name: String,
    }

    impl WCA {
        fn insert_person(&mut self, person: Person) {
            self.persons.insert(person.id.clone(), person);
        }

        fn visited_comp(&mut self, id: String, comp_id: String) {
            if self.competitions.contains_key(&id.clone()) {
            } else {
                self.competitions.insert(id.clone(), HashSet::new());
            }
            let possible_set = self.competitions.get_mut(&id);
            // TODO use unwrap, `None` not possible
            match possible_set {
                Some(set) => { set.insert(comp_id); },
                None => { println!("set not found"); },
            }
        }

        fn add_single_record(&mut self, id: String, puzzle: String, time: uint) {
            if self.records.contains_key(&id.clone()) {
            } else {
                self.records.insert(id.clone(), HashMap::new());
            }
            let map = self.records.get_mut(&id);
            match map {
                Some(map) => { map.insert(puzzle, Record{single: CompResult{time: time}, average: None}); },
                None => { println!("map not found"); },
            }
        }

        fn add_single_ranking(&mut self, puzzle_id: String, best: uint, competitor_id: String) {
            if self.single_rankings.contains_key(&puzzle_id) {
            } else {
                self.single_rankings.insert(puzzle_id.clone(), vec![]);
            }
            let vec = self.single_rankings.get_mut(&puzzle_id);
            match vec {
                Some(vec) => { vec.push(Ranking { result: CompResult {time: best}, competitor_id: competitor_id.clone()}); },
                None => { panic!("This will not happen") },
            }
        }

        fn add_average_ranking(&mut self, puzzle_id: String, best: uint, competitor_id: String) {
            if self.average_rankings.contains_key(&puzzle_id) {
            } else {
                self.average_rankings.insert(puzzle_id.clone(), vec![]);
            }
            let vec = self.average_rankings.get_mut(&puzzle_id);
            match vec {
                Some(vec) => { vec.push(Ranking { result: CompResult {time: best}, competitor_id: competitor_id.clone()}); },
                None => { panic!("This will not happen") },
            }
        }

        fn add_average_record(&mut self, id: String, puzzle: String, time: uint) {
            // This assumes that
            // a) Adding single records have been executed first.
            // b) For every average record exists one single record.
            let mut records = self.records.get_mut(&id).unwrap();
            let &mut record = records.get_mut(&puzzle).unwrap();

            records.insert(puzzle, Record { single: record.single, average: Some(CompResult{time: time}) });
        }

        pub fn number_of_comps(&mut self, id: &String) -> Option<uint> {
            self.competitions.get(id).map(|set| set.len())
        }

        pub fn find_competitor(&self, id: &String) -> Option<&Person> {
            self.persons.get(id)
        }

        pub fn find_competitors(&self, query: &String) -> Vec<&Person> {
            self.persons
                .lower_bound(query)
                .take_while(|t| t.val0().starts_with(query.as_slice()))
                .map(|t| t.val1())
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
                Single  => self.single_rankings.get(puzzle_id),
                Average => self.average_rankings.get(puzzle_id),
            }
        }

        pub fn new(persons_path: &Path, results_path: &Path, records_single_path: &Path, records_average_path: &Path, events_path: &Path) -> Box<WCA> {
            let mut w = box WCA { persons: TreeMap::new(), competitions: HashMap::new(), records: HashMap::new(), single_rankings: HashMap::new(), average_rankings: HashMap::new(), events: Vec::new() };
            load_persons(&mut *w, persons_path);
            load_competitions(&mut *w, results_path);
            load_single_records(&mut *w, records_single_path);
            load_average_records(&mut *w, records_average_path);
            load_events(&mut *w, events_path);
            w
        }
    }

    fn load_persons(w: &mut WCA, fp: &Path) {
        let mut rdr = csv::Reader::from_file(fp).has_headers(true).delimiter(b'\t');

        for record in rdr.decode() {
            let r: Person = record.unwrap();
            w.insert_person(r);
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
