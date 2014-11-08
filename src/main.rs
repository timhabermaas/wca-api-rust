extern crate nickel;
extern crate "wca-data" as w;
extern crate serialize;
extern crate http;

use std::sync::Arc;

use std::io::net::ip::Ipv4Addr;
use nickel::{ Halt, Nickel, Request, QueryString, Response, HttpRouter, RequestHandler, MiddlewareResult };
use w::wca_data;
use std::collections::TreeMap;
use serialize::json;
use serialize::json::ToJson;
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

struct Competitor {
    id: String,
    name: String,
    gender: wca_data::Gender,
}

#[deriving(Encodable)]
struct CompetitorPartOfCollection<'a> {
    id: &'a str,
    name: &'a str,
    gender: &'a str,
}

impl ToJson for Competitor {
    fn to_json(&self) -> json::Json {
        let mut sub = TreeMap::new();
        sub.insert("id".to_string(), self.id.to_json());
        sub.insert("name".to_string(), self.name.to_json());
        let gender: Option<String> = match self.gender {
            wca_data::Male   => Some("m".to_string()),
            wca_data::Female => Some("f".to_string()),
            _                => None,
        };
        sub.insert("gender".to_string(), gender.to_json());

        let mut d = TreeMap::new();
        d.insert("competitor".to_string(), json::Object(sub));
        json::Object(d)
    }
}



impl RequestHandler for CompetitorHandler {
    fn handle(&self, req: &Request, res: &mut Response) -> MiddlewareResult {
        let id = req.param("id");
        let result = self.data.find_competitor(id.to_string());

        res.origin.headers.content_type = Some(MediaType::new("application".into_string(),
                                                              "json".into_string(),
                                                              vec![("charset".into_string(),
                                                              "utf8".into_string())]));

        match result {
            Some(c) => {
                let c = Competitor { id: c.id.clone(), name: c.name.clone(), gender: c.gender };
                let data = c.to_json().to_string();
                res.send(data);
            },
            None => {
                res.status_code(status::NotFound);
                res.send("{\"error\": \"not found\"}");
            }
        }

        Ok(Halt)
    }
}

fn gender_to_str(gender: &wca_data::Gender) -> &str {
    match gender {
        &wca_data::Male   => "m",
        &wca_data::Female => "f",
        _                 => "",
    }
}

impl RequestHandler for CompetitorSearchHandler {
    fn handle(&self, req: &Request, res: &mut Response) -> MiddlewareResult {
        let q = req.query("q", "default");
        let m = q.get(0).unwrap();
        let competitors = self.data.find_competitors(m.clone());
        let competitors: Vec<CompetitorPartOfCollection> = competitors.iter().map(|c| CompetitorPartOfCollection { id: c.id.as_slice(), name: c.name.as_slice(), gender: gender_to_str(&c.gender) }).collect();

        res.origin.headers.content_type = Some(MediaType::new("application".into_string(),
                                                              "json".into_string(),
                                                              vec![("charset".into_string(),
                                                              "utf8".into_string())]));
        res.send(format!("{}", json::encode(&competitors)));
        Ok(Halt)
    }
}

impl RequestHandler for CompetitorRecordsHandler {
    fn handle(&self, req: &Request, res: &mut Response) -> MiddlewareResult {
        let id = req.param("id");
        match self.data.find_records(id.to_string()) {
            Some(r) => {
                res.send(json::encode(r));
            },
            None => {
                res.status_code(status::NotFound);
                res.send("{\"error\": \"not found\"}");
            }
        }

        Ok(Halt)
    }
}


fn main() {
    let mut server = Nickel::new();
    let mut router = Nickel::router();
    println!("Importing");
    let w = wca_data::build_from_files(&Path::new("./data/WCA_export_Persons.tsv"), &Path::new("./data/WCA_export_Results.tsv"), &Path::new("./data/WCA_export_RanksSingle.tsv"), &Path::new("./data/WCA_export_RanksAverage.tsv"));
    println!("Importing Done");

    let w_arc = Arc::new(*w);

    router.get("/competitors/:id", CompetitorHandler { data: w_arc.clone() });
    router.get("/competitors/:id/records", CompetitorRecordsHandler { data: w_arc.clone() });
    router.get("/competitors", CompetitorSearchHandler { data: w_arc.clone() });

    server.utilize(Nickel::query_string());
    server.utilize(router);

    server.listen(Ipv4Addr(0, 0, 0, 0), 6767);
}
