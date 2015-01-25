extern crate nickel;
extern crate "wca-data" as w;
extern crate http;
extern crate "rustc-serialize" as rustc_serialize;

use std::sync::Arc;

use std::old_io::net::ip::Ipv4Addr;
use nickel::{ Halt, Nickel, Request, QueryString, Response, HttpRouter, Middleware, MiddlewareResult };
use w::wca_data;
use std::collections::BTreeMap;
use rustc_serialize::json::{mod, Json, ToJson};
use http::status;
use http::headers::content_type::MediaType;

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
    competition_count: uint,
}

#[derive(RustcEncodable, PartialEq)]
struct CompetitorPartOfCollection<'a> {
    id: &'a str,
    name: &'a str,
    gender: &'a str,
    country: &'a str,
    competition_count: uint,
}

#[derive(RustcEncodable)]
struct Ranking<'a> {
    time: uint,
    competitor: CompetitorPartOfCollection<'a>,
}

impl ToJson for Competitor {
    fn to_json(&self) -> Json {
        let mut sub = BTreeMap::new();
        sub.insert("id".to_string(), self.id.to_json());
        sub.insert("name".to_string(), self.name.to_json());
        sub.insert("competition_count".to_string(), self.competition_count.to_json());
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



impl Middleware for CompetitorHandler {
    fn invoke(&self, req: &mut Request, res: &mut Response) -> MiddlewareResult {
        let id = req.param("id");
        let result = self.data.find_competitor(&id.to_string());

        res.origin.headers.content_type = Some(MediaType::new("application".to_string(),
                                                              "json".to_string(),
                                                              vec![("charset".to_string(),
                                                              "utf8".to_string())]));

        match result {
            Some(c) => {
                let c = Competitor { id: c.id.clone(), name: c.name.clone(), gender: c.gender.clone(), competition_count: c.competition_count };
                let data = c.to_json().to_string();
                res.send(data);
            },
            None => {
                res.status_code(status::NotFound);
                res.send("{\"error\": \"not found\"}");
            },
        }

        Ok(Halt)
    }
}

impl Middleware for RecordsHandler {
    fn invoke(&self, req: &mut Request, res: &mut Response) -> MiddlewareResult {
        res.origin.headers.content_type = Some(MediaType::new("application".to_string(),
                                                              "json".to_string(),
                                                              vec![("charset".to_string(),
                                                              "utf8".to_string())]));

        let puzzle = req.param("puzzle_id");
        let type_   = req.param("type");
        let rankings = match type_ {
            "single"  => self.data.find_rankings(&puzzle.to_string(), wca_data::ResultType::Single),
            "average" => self.data.find_rankings(&puzzle.to_string(), wca_data::ResultType::Average),
            _         => { res.status_code(status::NotFound); return Ok(Halt); }
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
                res.send(json::encode(&rankings).unwrap());
            },
            None => {
                res.status_code(status::NotFound);
                res.send("{\"error\": \"not found\"}");
            },
        }

        Ok(Halt)
    }
}

fn gender_to_str(gender: &wca_data::Gender) -> &str {
    match gender {
        &wca_data::Gender::Male   => "m",
        &wca_data::Gender::Female => "f",
        _                 => "",
    }
}

impl Middleware for CompetitorSearchHandler {
    fn invoke(&self, req: &mut Request, res: &mut Response) -> MiddlewareResult {
        let q = req.query("q", "default");
        let m = q.get(0).unwrap();
        let competitors = self.data.find_competitors(m);
        let competitors: Vec<CompetitorPartOfCollection> = competitors.iter().map(|c| CompetitorPartOfCollection { id: c.id.as_slice(), name: c.name.as_slice(), gender: gender_to_str(&c.gender), country: c.country.as_slice(), competition_count: c.competition_count }).collect();
        let mut wrapped_competitors: BTreeMap<String, &Vec<CompetitorPartOfCollection>> = BTreeMap::new();
        wrapped_competitors.insert("competitors".to_string(), &competitors);

        res.origin.headers.content_type = Some(MediaType::new("application".to_string(),
                                                              "json".to_string(),
                                                              vec![("charset".to_string(),
                                                              "utf8".to_string())]));
        res.send(json::encode(&wrapped_competitors).unwrap());
        Ok(Halt)
    }
}

impl Middleware for CompetitorRecordsHandler {
    fn invoke(&self, req: &mut Request, res: &mut Response) -> MiddlewareResult {
        res.origin.headers.content_type = Some(MediaType::new("application".to_string(),
                                                              "json".to_string(),
                                                              vec![("charset".to_string(),
                                                              "utf8".to_string())]));

        let id = req.param("id");
        match self.data.find_records(&id.to_string()) {
            Some(r) => {
                res.send(json::encode(r).unwrap());
            },
            None => {
                res.status_code(status::NotFound);
                res.send("{\"error\": \"not found\"}");
            }
        }

        Ok(Halt)
    }
}

impl Middleware for EventsHandler {
    fn invoke(&self, _: &mut Request, res: &mut Response) -> MiddlewareResult {
        res.origin.headers.content_type = Some(MediaType::new("application".to_string(),
                                                              "json".to_string(),
                                                              vec![("charset".to_string(),
                                                              "utf8".to_string())]));
        res.send(json::encode(self.data.find_events()).unwrap());

        Ok(Halt)
    }
}

impl Middleware for SelectiveRecordsHandler {
    fn invoke(&self, req: &mut Request, res: &mut Response) -> MiddlewareResult {
        res.origin.headers.content_type = Some(MediaType::new("application".to_string(),
                                                              "json".to_string(),
                                                              vec![("charset".to_string(),
                                                              "utf8".to_string())]));

        let mut puzzle_id = String::new();
        {
            let req2 = &req;
            puzzle_id = req2.param("puzzle_id").to_string();
        }
        let ids = req.query("ids", "");
        let records = self.data.find_rankings_for(&puzzle_id.to_string(), ids.into_owned());

        res.send(json::encode(&records).unwrap());

        Ok(Halt)
    }
}


fn main() {
    let mut server = Nickel::new();
    let mut router = Nickel::router();
    println!("Importing");
    let w = wca_data::build_from_files(&Path::new("./data/WCA_export_Persons.tsv"),
                                       &Path::new("./data/WCA_export_Results.tsv"),
                                       &Path::new("./data/WCA_export_RanksSingle.tsv"),
                                       &Path::new("./data/WCA_export_RanksAverage.tsv"),
                                       &Path::new("./data/WCA_export_Events.tsv"));
    println!("Importing Done");

    let w_arc = Arc::new(*w);

    router.get("/competitors/:id", CompetitorHandler { data: w_arc.clone() });
    router.get("/competitors/:id/records", CompetitorRecordsHandler { data: w_arc.clone() });
    router.get("/competitors", CompetitorSearchHandler { data: w_arc.clone() });
    router.get("/records/:puzzle_id/:type", RecordsHandler { data: w_arc.clone() });
    router.get("/records/:puzzle_id", SelectiveRecordsHandler { data: w_arc.clone() });
    router.get("/events", EventsHandler { data: w_arc.clone() });

    server.utilize(router);

    server.listen(Ipv4Addr(0, 0, 0, 0), 6767);
}
