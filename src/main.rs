extern crate "wca-data" as w;
extern crate "rustc-serialize" as rustc_serialize;
extern crate iron;
extern crate router;

use std::sync::Arc;

use w::wca_data;
use std::collections::BTreeMap;
use rustc_serialize::json;
use rustc_serialize::json::{Json, ToJson};

use iron::{Iron, Handler, Request, Response, IronResult};
use iron::status;
use router::{Router, Params};


struct CompetitorHandler {
    data: Arc<wca_data::WCA>,
}

struct CompetitorSearchHandler {
    data: Arc<wca_data::WCA>,
}

struct CompetitorRecordsHandler {
    data: Arc<wca_data::WCA>,
}

struct RecordsHandler {
    data: Arc<wca_data::WCA>,
}

struct EventsHandler {
    data: Arc<wca_data::WCA>,
}

struct SelectiveRecordsHandler {
    data: Arc<wca_data::WCA>,
}

struct Competitor {
    id: String,
    name: String,
    gender: wca_data::Gender,
    country: String,
    competition_count: u32,
}

#[derive(RustcEncodable, PartialEq)]
struct CompetitorPartOfCollection<'a> {
    id: &'a str,
    name: &'a str,
    gender: &'a str,
    country: &'a str,
    competition_count: u32,
}

#[derive(RustcEncodable)]
struct Ranking<'a> {
    time: u32,
    competitor: CompetitorPartOfCollection<'a>,
}

impl ToJson for Competitor {
    fn to_json(&self) -> Json {
        let mut sub = BTreeMap::new();
        sub.insert("id".to_string(), self.id.to_json());
        sub.insert("name".to_string(), self.name.to_json());
        sub.insert("competition_count".to_string(), self.competition_count.to_json());
        sub.insert("country".to_string(), self.country.to_json());
        let gender: Option<String> = match self.gender {
            wca_data::Gender::Male   => Some("m".to_string()),
            wca_data::Gender::Female => Some("f".to_string()),
            _                        => None,
        };
        sub.insert("gender".to_string(), gender.to_json());

        let mut d = BTreeMap::new();
        d.insert("competitor".to_string(), Json::Object(sub));
        Json::Object(d)
    }
}

fn gender_to_str(gender: &wca_data::Gender) -> &str {
    match gender {
        &wca_data::Gender::Male   => "m",
        &wca_data::Gender::Female => "f",
        _                 => "",
    }
}

impl Handler for CompetitorHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let ref id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
        let result = self.data.find_competitor(&id.to_string()).unwrap();
        let c = Competitor { id: result.id.clone(), name: result.name.clone(), gender: result.gender.clone(), competition_count: result.competition_count, country: result.country.clone() };
        let data = c.to_json().to_string();
        Ok(Response::with((status::Ok, data)))
    }
}

impl Handler for CompetitorSearchHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // TODO use a proper way to parse query strings
        let query = req.url.clone().query.unwrap();
        let q = &query[2..query.len()];

        let competitors = self.data.find_competitors(&q.to_string());
        let competitors: Vec<CompetitorPartOfCollection> = competitors.iter().map(|c| CompetitorPartOfCollection { id: c.id.as_slice(), name: c.name.as_slice(), gender: gender_to_str(&c.gender), country: c.country.as_slice(), competition_count: c.competition_count }).collect();
        let mut wrapped_competitors: BTreeMap<String, &Vec<CompetitorPartOfCollection>> = BTreeMap::new();
        wrapped_competitors.insert("competitors".to_string(), &competitors);

        Ok(Response::with((status::Ok, json::encode(&wrapped_competitors).unwrap())))
    }
}

impl Handler for CompetitorRecordsHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {

        let ref id = req.extensions.get::<Router>().unwrap().find("id").unwrap();

        match self.data.find_records(&id.to_string()) {
            Some(r) => {
                Ok(Response::with((status::Ok, json::encode(r).unwrap())))
            },
            None => {
                Ok(Response::with((status::NotFound, "{\"error\": \"not found\"}")))
            }
        }

    }
}

impl Handler for RecordsHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let ref puzzle = req.extensions.get::<Router>().unwrap().find("puzzle_id").unwrap();
        let ref _type = req.extensions.get::<Router>().unwrap().find("type").unwrap();
        let rankings = match *_type {
            "single"  => self.data.find_rankings(&puzzle.to_string(), wca_data::ResultType::Single),
            "average" => self.data.find_rankings(&puzzle.to_string(), wca_data::ResultType::Average),
            _         => { return Ok(Response::with((status::NotFound, ""))); }
        };
        match rankings {
            Some(v) => {
                let rankings: Vec<Ranking> = v.iter().map(|r| {
                    let competitor = self.data.find_competitor(&r.competitor_id).unwrap();
                    Ranking {
                        time: r.result.time,
                        competitor: CompetitorPartOfCollection {
                            id: competitor.id.as_slice(),
                            name: competitor.name.as_slice(),
                            gender: gender_to_str(&competitor.gender),
                            country: competitor.country.as_slice(),
                            competition_count: competitor.competition_count,
                        }
                    }
                }
                ).collect();
                Ok(Response::with((status::Ok, json::encode(&rankings).unwrap())))
            },
            None => {
                Ok(Response::with((status::NotFound, "")))
            },
        }
    }
}

// TODO add Middleware for setting JSON response and UTF-8 encoding

fn main() {
    println!("Importing");
    let w = wca_data::build_from_files(&Path::new("./data/WCA_export_Persons.tsv"),
                                       &Path::new("./data/WCA_export_Results.tsv"),
                                       &Path::new("./data/WCA_export_RanksSingle.tsv"),
                                       &Path::new("./data/WCA_export_RanksAverage.tsv"),
                                       &Path::new("./data/WCA_export_Events.tsv"));
    println!("Importing Done");

    let w_arc = Arc::new(*w);

    let mut router = Router::new();

    router.get("/competitors", CompetitorSearchHandler { data: w_arc.clone() });
    router.get("/competitors/:id", CompetitorHandler { data: w_arc.clone() });
    router.get("/competitors/:id/records", CompetitorRecordsHandler { data: w_arc.clone() });
    router.get("/records/:puzzle_id/:type", RecordsHandler { data: w_arc.clone() });

    Iron::new(router).listen("localhost:3000").unwrap();
}
