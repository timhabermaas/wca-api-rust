extern crate "wca-data" as w;
extern crate "rustc-serialize" as rustc_serialize;
extern crate iron;
extern crate router;

use std::sync::Arc;

use w::wca_data;
use std::collections::BTreeMap;
use rustc_serialize::json;
use rustc_serialize::json::{Json, ToJson};
use std::path::Path;

use iron::{Iron, Chain, Handler, Request, Response, IronResult, AfterMiddleware};
use iron::status;
use iron::headers;
use iron::mime::Mime;
use router::{Router};


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
        match self.data.find_competitor(&id.to_string()) {
            Option::Some(result) => {
                let c = Competitor { id: result.id.clone(), name: result.name.clone(), gender: result.gender.clone(), competition_count: result.competition_count, country: result.country.clone() };
                let data = c.to_json().to_string();
                Ok(Response::with((status::Ok, data)))
            },
            None => {
                Ok(Response::with((status::NotFound, "{\"error\": \"not found\"}")))
            },
        }
    }
}

impl Handler for CompetitorSearchHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // TODO use a proper way to parse query strings
        let query = req.url.query.clone().unwrap();
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

impl Handler for EventsHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, json::encode(self.data.find_events()).unwrap())))
    }
}

impl Handler for SelectiveRecordsHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let ref puzzle_id = req.extensions.get::<Router>().unwrap().find("puzzle_id").unwrap();
        // parsing of ids= query parameters
        let foo = req.url.query.clone().unwrap();
        let ids: Vec<String> = foo.split('&').map(|param| (&param[4..param.len()]).to_string()).collect();

        let records = self.data.find_rankings_for(&puzzle_id.to_string(), ids);
        Ok(Response::with((status::Ok, json::encode(&records).unwrap())))
    }

}

struct JSONAcceptHeaderMiddleware;

impl AfterMiddleware for JSONAcceptHeaderMiddleware {
    fn after(&self, _: &mut Request, res: Response) -> IronResult<Response> {
        let mut response = res;
        let mime: Mime = "application/json;charset=utf-8".parse().unwrap();
        response.headers.set(headers::ContentType(mime));
        Ok(response)
    }
}

fn main() {
    println!("Importing");
    let w = wca_data::build_from_files(Path::new("./data/WCA_export_Persons.tsv"),
                                       Path::new("./data/WCA_export_Results.tsv"),
                                       Path::new("./data/WCA_export_RanksSingle.tsv"),
                                       Path::new("./data/WCA_export_RanksAverage.tsv"),
                                       Path::new("./data/WCA_export_Events.tsv"));
    println!("Importing Done");

    let w_arc = Arc::new(*w);

    let mut router = Router::new();

    router.get("/competitors", CompetitorSearchHandler { data: w_arc.clone() });
    router.get("/competitors/:id", CompetitorHandler { data: w_arc.clone() });
    router.get("/competitors/:id/records", CompetitorRecordsHandler { data: w_arc.clone() });
    router.get("/records/:puzzle_id/:type", RecordsHandler { data: w_arc.clone() });
    router.get("/records/:puzzle_id/", SelectiveRecordsHandler { data: w_arc.clone() });
    router.get("/events", EventsHandler { data: w_arc.clone() });

    let mut chain = Chain::new(router);

    chain.link_after(JSONAcceptHeaderMiddleware);

    Iron::new(chain).http("0.0.0.0:3000").unwrap();
}
