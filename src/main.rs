extern crate iron;
extern crate router;
extern crate serialize;
extern crate "wca-data" as w;

use std::io::net::ip::Ipv4Addr;
use iron::{status, Handler, Iron, Request, Response, IronResult};
use router::{Router, Params};
use w::wca_data;
use std::collections::TreeMap;
use serialize::json;
use serialize::json::ToJson;

struct CompetitorHandler {
    data: wca_data::WCA,
}

struct CompetitorRecordsHandler {
    data: wca_data::WCA,
}

struct Competitor {
    id: String,
    name: String,
}

impl ToJson for Competitor {
    fn to_json(&self) -> json::Json {
        let mut sub = TreeMap::new();
        sub.insert("id".to_string(), self.id.to_json());
        sub.insert("name".to_string(), self.name.to_json());

        let mut d = TreeMap::new();
        d.insert("competitor".to_string(), json::Object(sub));
        json::Object(d)
    }
}

impl Handler for CompetitorRecordsHandler {
    fn call(&self, req: &mut Request) -> IronResult<Response> {
        Ok(Response::with(status::Ok, "bla"))
    }
}

impl Handler for CompetitorHandler {
    fn call(&self, req: &mut Request) -> IronResult<Response> {
        let id: &str = req.extensions.find::<Router, Params>().unwrap().find("id").unwrap();
        // TODO extract some of it into a function which takes an Option<T> and returns either a
        // valid json object or a 404 not found response
        match self.data.find_competitor(String::from_str(id)) {
            Some(c) => {
                let encoded = Competitor { id: c.id.clone(), name: c.name.clone() }.to_json().to_string();
                Ok(Response::with(status::Ok, encoded))
            },
            None   => {
                let encoded = String::from_str("not found");
                Ok(Response::with(status::NotFound, encoded))
            }
        }
    }
}


fn main() {
    println!("Importing");
    let w = wca_data::build_from_files(&Path::new("./data/WCA_export_Persons.tsv"), &Path::new("./data/WCA_export_Results.tsv"), &Path::new("./data/WCA_export_RanksAverage.tsv"));
    println!("Done importing");
    let mut router = Router::new();
    router.get("/competitors/:id", CompetitorHandler{data: *w});
    //router.get("/competitors/:id/records", CompetitorRecordsHandler{data: & *w});
    let server = Iron::new(router);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
