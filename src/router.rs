use http::server::{Request, ResponseWriter};
use regex::Regex;
use std::collections::hashmap::HashMap;
use request;

/// A Route is the basic data structure that stores both the path
/// and the handler that gets executed for the route.
/// The path can contain variable pattern such as `user/:userid/invoices`
struct Route {
    pub path: String,
    pub handler: fn(request: request::Request, response: &mut ResponseWriter),
    pub variables: HashMap<String, uint>,
    matcher: Regex
}

impl Clone for Route {
    fn clone(&self) -> Route {
        Route { 
            path: self.path.clone(), 
            handler: self.handler, 
            matcher: self.matcher.clone(),
            variables: self.variables.clone() 
        }
    }
}

/// A RouteResult is what the router returns when `match_route` is called.
/// It contains the matched `route` and also a `params` property holding
/// a HashMap with the keys being the variable names and the value being the
/// evaluated string
struct RouteResult<'a> {
    pub route: &'a Route,
    pub params: HashMap<String, String>
}

/// The PathUtils collects some small helper methods that operate on the path
struct PathUtils;

static REGEX_VAR_SEQ: Regex            = regex!(r":([a-zA-Z0-9_-]*)");
static VARIABLE_SEQUENCE:&'static str  = "(.[a-zA-Z0-9_-]*)";
static REGEX_START:&'static str        = "^";
static REGEX_END:&'static str          = "$";


impl PathUtils {
    fn create_regex (route_path: &str) -> Regex {

        let result = REGEX_START.to_string()
                                .append(REGEX_VAR_SEQ.replace_all(route_path, VARIABLE_SEQUENCE).as_slice())
                                .append(REGEX_END);

        match Regex::new(result.as_slice()) {
            Ok(re) => re,
            Err(err) => fail!("{}", err)
        }
    }

    fn get_variable_info (route_path: &str) -> HashMap<String, uint> {
        REGEX_VAR_SEQ.captures_iter(route_path)
             .enumerate()
             .map(|(i, matched)| (matched.at(1).to_string(), i))
             .collect()
    }
}

/// The Router's job is it to hold routes and to resolve them later against
/// concrete URLs

#[deriving(Clone)]
pub struct Router{
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new () -> Router {
        Router {
            routes: Vec::new()
        }
    }

    pub fn add_route (&mut self, path: String, handler: fn(request: request::Request, response: &mut ResponseWriter)) -> () {
        let matcher = PathUtils::create_regex(path.as_slice());
        let variable_infos = PathUtils::get_variable_info(path.as_slice());
        let route = Route {
            path: path,
            matcher: matcher,
            handler: handler,
            variables: variable_infos
        };
        self.routes.push(route);
    }

    pub fn match_route<'a>(&'a self, path: String) -> Option<RouteResult<'a>> {
        let route = self.routes.iter().find(|item| item.matcher.is_match(path.as_slice()));

        // can we improve on all this nested stuff? Is this the intended way to handle it?
        match route {
            Some(r) => {
                match r.matcher.captures(path.as_slice()) {
                    Some(captures) => {
                        let mut map = HashMap::new();
                        for (name, pos) in r.variables.iter() {
                            map.insert(name.to_string(), captures.at(pos + 1).to_string());
                        }

                        Some(RouteResult {
                            route: r,
                            params: map
                        })
                    },
                    None => Some(RouteResult{
                        route: r,
                        params: HashMap::new()
                    })
                }
            },
            None => None
        }
    }
}


#[test]
fn creates_map_with_var_variable_infos () {
    let map = PathUtils::get_variable_info("foo/:uid/bar/:groupid");
    
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&"uid".to_string()), &0);
    assert_eq!(map.get(&"groupid".to_string()), &1);
}

#[test]
fn creates_regex_with_captures () {
    let regex = PathUtils::create_regex("foo/:uid/bar/:groupid");
    assert_eq!(regex.is_match("foo/4711/bar/5490"), true);

    let caps = regex.captures("foo/4711/bar/5490").unwrap();

    assert_eq!(caps.at(1), "4711");
    assert_eq!(caps.at(2), "5490");
    assert_eq!(regex.is_match("foo/"), false);
}

#[test]
fn can_match_var_routes () {
    let route_store = &mut Router::new();

    fn handler (request: request::Request, response: &mut ResponseWriter) -> () {
        response.write("hello from foo".as_bytes()); 
    };

    route_store.add_route("/foo/:userid".to_string(), handler);
    route_store.add_route("/bar".to_string(), handler);
    
    let route_result = route_store.match_route("/foo/4711".to_string()).unwrap();
    let route = route_result.route;

    assert_eq!(route_result.params.get(&"userid".to_string()), &"4711".to_string());

    //assert the route has identified the variable
    assert_eq!(route.variables.len(), 1);
    assert_eq!(route.variables.get(&"userid".to_string()), &0);


    let route_result = route_store.match_route("/bar/4711".to_string());

    let result = match route_result {
        Some(res) => true,
        None => false
    };

    assert_eq!(result, false);

    let route_result = route_store.match_route("/foo".to_string());

    let result = match route_result{
        Some(res) => true,
        None => false
    };

    assert_eq!(result, false);
}