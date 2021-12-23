use rocket::{
    fairing::{self, Fairing, Info, Kind},
    http::Method,
    route::{Handler, Outcome},
    Build, Data, Request, Response, Rocket,
};
use std::collections::HashMap;

/// This fairing implements the HTTP `OPTION` method.
/// The response will include `Access-Control-Allow-Methods` with a list of allowed methods.
pub struct OptionsFairing;

#[rocket::async_trait]
impl Fairing for OptionsFairing {
    fn info(&self) -> Info {
        Info {
            name: "OPTIONS",
            kind: Kind::Ignite | Kind::Response,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let mut result: HashMap<String, Vec<Method>> = HashMap::new();
        for route in rocket.routes() {
            let path = route.uri.path().to_string();
            let methods = result.entry(path).or_default(); // all methods or empty Vec
            if !methods.contains(&route.method) {
                methods.push(route.method);
            }
        }
        let options_routes: Vec<rocket::Route> = result
            .into_iter()
            .filter(|(_, methods)| !methods.contains(&Method::Options))
            .map(|(path, methods)| get_route(&path, methods))
            .collect();
        // Add all the `OPTION` routes
        let rocket = rocket.mount("/", options_routes);
        Ok(rocket)
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_raw_header("Access-Control-Allow-Origin", "*");
        res.set_raw_header("Vary", "Origin");
    }
}

fn get_route(path: &str, methods: Vec<Method>) -> rocket::Route {
    let handler = OptionsHandler { methods };
    rocket::Route::new(Method::Options, path, handler)
}

#[derive(Clone)]
struct OptionsHandler {
    methods: Vec<Method>,
}

#[rocket::async_trait]
impl Handler for OptionsHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, _data: Data<'r>) -> Outcome<'r> {
        Outcome::from(
            req,
            OptionsHandler {
                methods: self.methods.clone(),
            },
        )
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for OptionsHandler {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let methods = self
            .methods
            .into_iter()
            .map(Method::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        rocket::response::Response::build()
            .raw_header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization",
            )
            .raw_header("Access-Control-Allow-Methods", methods)
            .raw_header("Access-Control-Allow-Origin", "*")
            .ok()
    }
}
