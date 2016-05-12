#[macro_use]
extern crate rustless;

extern crate iron;
extern crate url;
extern crate rustc_serialize;
extern crate valico;
extern crate chrono;
extern crate uuid;
extern crate regex;
extern crate crypto;
extern crate urlencoded;


use std::collections::{BTreeMap};
use std::sync::{Arc, Mutex};
use rustc_serialize::base64::{STANDARD, ToBase64};
use uuid::Uuid;

use std::fmt;
use std::error;
use std::error::Error as StdError;
use valico::json_dsl;

use std::error::Error;
use rustless::server::status;
use rustless::errors::Error as RError;
use rustless::batteries::swagger;
use rustless::{Nesting};
use rustc_serialize::json;
use rustc_serialize::json::{ToJson, Json};
use crypto::sha2;
use crypto::digest::Digest;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use regex::Regex;

#[derive(Clone, Debug, RustcDecodable)]
#[allow(non_snake_case)] // fuck you type
struct User {
    name: String,
    id: String,
    mail: Option<String>,
    created: String,
    signature : String
}


// Specify encoding method manually
impl ToJson for User {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        // All standard types implement `to_json()`, so use it
        d.insert("type".to_string(), "user".to_json());
        d.insert("name".to_string(), self.name.to_json());
        d.insert("id".to_string(), self.id.to_json());
        d.insert("mail".to_string(), self.mail.to_json());
        d.insert("created".to_string(), self.created.to_json());
        d.insert("signature".to_string(), self.signature.to_json());
        Json::Object(d)
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {:?} {} {}", self.name, self.id, self.mail, self.signature, self.created)
    }
}

#[derive(Clone, Debug, RustcDecodable)]
struct Game {
    name: String,
    id: String,
    url: Option<String>,
    signature: String,
    created: String
}

// Specify encoding method manually
impl ToJson for Game {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        // All standard types implement `to_json()`, so use it
        d.insert("type".to_string(), "game".to_json());
        d.insert("name".to_string(), self.name.to_json());
        d.insert("id".to_string(), self.id.to_json());
        d.insert("url".to_string(), self.url.to_json());
        d.insert("created".to_string(), self.created.to_json());
        d.insert("signature".to_string(), self.signature.to_json());
        Json::Object(d)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {:?} {} {}", self.name, self.id, self.url, self.signature, self.created)
    }
}

#[derive(Clone, Debug, RustcDecodable)]
struct Gamestate {
    gameid: String,
    userid: String,
    created: String,
    state: String
}

// Specify encoding method manually
impl ToJson for Gamestate {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        // All standard types implement `to_json()`, so use it
        d.insert("type".to_string(), "gamestate".to_json());
        d.insert("gameid".to_string(), self.gameid.to_json());
        d.insert("userid".to_string(), self.userid.to_json());
        d.insert("created".to_string(), self.created.to_json());
        d.insert("state".to_string(), self.state.to_json());
        Json::Object(d)
    }
}

impl fmt::Display for Gamestate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.gameid, self.userid, self.created, self.state)
    }
}

#[derive(Clone, Debug, RustcDecodable)]
struct GameKey {
    users: Vec<User>,
    games: Vec<Game>,
    gamestates: Vec<Gamestate>
}

// Specify encoding method manually
impl ToJson for GameKey {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        // All standard types implement `to_json()`, so use it
        d.insert("users".to_string(), self.users.to_json());
        d.insert("games".to_string(), self.games.to_json());
        d.insert("gamestates".to_string(), self.gamestates.to_json());
        Json::Object(d)
    }
}

impl fmt::Display for GameKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?} {:?}", self.users, self.games, self.gamestates)
    }
}

#[derive(Debug)]
pub struct InvalidMail;

impl error::Error for InvalidMail {
    fn description(&self) -> &str {
        return "InvalidMail";
    }
}


impl fmt::Display for InvalidMail {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(formatter)
    }
}

#[derive(Debug)]
pub struct UnauthorizedError;

impl error::Error for UnauthorizedError {
    fn description(&self) -> &str {
        return "UnauthorizedError";
    }
}


impl fmt::Display for UnauthorizedError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(formatter)
    }
}

#[derive(Debug)]
pub struct NotFoundError;

impl error::Error for NotFoundError {
    fn description(&self) -> &str {
        return "NotFoundError";
    }
}


impl fmt::Display for NotFoundError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(formatter)
    }
}

fn auth_signature(id: String, pwd: String) -> String {

    // create a Sha256 object
    let mut hasher = sha2::Sha256::new();
    // write input message
    let in_str = id + "," + &pwd;

    // println!("auth_signature call: in_str {}", &in_str);

    hasher.input_str(&in_str);
    // read hash digest
    // let hex = hasher.result_str();

    // let mut bytes = vec!(hasher.output_bytes(), u8);
    let mut bytes = Vec::new();
    bytes.resize(hasher.output_bytes(), 0u8);

    hasher.result(bytes.as_mut_slice());

    // println!("auth_signature: hex {}\nsha256 {:?}", hex, bytes);

    let base = bytes.to_base64(STANDARD);

    base
}


fn create_gamekey() -> GameKey {

    let users: Vec<User> = Vec::new();
    let games: Vec<Game> = Vec::new();
    let gamestates: Vec<Gamestate> = Vec::new();

    GameKey {users: users, games: games, gamestates: gamestates}

}

fn get_gamekey() -> GameKey {

    let path = Path::new("foo.txt");
    let display = path.display();

    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => {
            println!("couldn't open {}: {}", display, Error::description(&why));
            return create_gamekey();
        },
        Ok(file) => {
            file
        },
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                   Error::description(&why)),
        Ok(_) => println!("{} contains:\n{}", display, s),
    }

    let gk: GameKey = match json::decode(s.as_str()) {
        Ok(s) => {
            s
        },
        Err(err) => {
            println!("gk match err: {}", err);
            create_gamekey()
        }
    };
    println!("gk match: {}", gk);

    return gk;

}

fn save_gamekey(gk: GameKey) {
    let js = &gk.to_json().to_string();
    println!("\nsave_gamekey got:\n{}\n", &js);

    let path = Path::new("foo.txt");
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => {
            panic!("couldn't create {}: {}", display, Error::description(&why))
        }

        Ok(file) => {
            file
        }
    };

    match file.write_all(js.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to {}: {}", display, Error::description(&why))
        },
        Ok(_) => {
            println!("successfully wrote to {}", display);
        }
    }
}

fn get_user_by_id(list: Vec<User>, id: &str) -> Option<User> {
    println!("get_user_by_id called with {} \n {:?}", id, list);
    for e in list {
        println!("\nget_user_by_id: e {} id {}", e, id);
        if e.id == id {
            println!("{}", e);
            return Some(e)
        }
    }
    None
}


fn get_user_by_name(list: Vec<User>, id: &str) -> Option<User> {
    for e in list {
        println!("\nget_user_by_name: e \"{}\" id \"{}\"", e, id);
        if e.name == id {
            println!("{}", e);
            return Some(e)
        }
    }
    None
}

fn main() {

    let storage = Arc::new(Mutex::new(get_gamekey()));

    let app = rustless::Application::new(rustless::Api::build(|api| {

        api.mount(swagger::create_api("api-docs"));

        api.error_formatter(|err, _media| {
            match err.downcast::<InvalidMail>() {
                Some(_) => {
                    return Some(rustless::Response::from( status::StatusCode::BadRequest, Box::new("Invalid mail!") ))
                },
                None => match err.downcast::<UnauthorizedError>() {
                    Some(_) => {
                        Some(rustless::Response::from(status::StatusCode::Unauthorized, Box::new("Unauthorized!")))
                    },
                    None => match err.downcast::<NotFoundError>() {
                        Some(_) => {
                            Some(rustless::Response::from(status::StatusCode::NotFound, Box::new("Not found!")))
                        },
                        None => None
                    }
                }
            }
        });

        let storage_clone = storage.clone();
        api.get("users", |endpoint| {
            endpoint.summary("Lists all registered users");
            endpoint.desc("");
            endpoint.handle(move |client, _| {
                let users: Vec<User> = storage_clone.lock().unwrap().users.clone();

                let user_json = &users.to_json();

                println!("get user: {:?}", user_json.to_string());

                client.json( &user_json )

                // client.json(&test)
            })
        });

        let storage_clone = storage.clone();
        api.post("user", |endpoint| {
            endpoint.summary("Creates a user");
            endpoint.desc("Use this to create a user");
            endpoint.params(|params| {
                params.req_typed("name", json_dsl::string());
                params.req_typed("pwd", json_dsl::string());
                params.opt_typed("mail", json_dsl::string());
            });

            endpoint.handle(move |mut client, params| {
                let message_object = params.as_object().unwrap();

                let new_name = message_object.get("name").unwrap().as_string().unwrap().to_string().replace("+", " "); // why...
                let new_pwd  = message_object.get("pwd").unwrap().as_string().unwrap().to_string();
                let new_created = chrono::UTC::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
                let new_id = Uuid::new_v4().to_string();
                let new_sig = auth_signature(new_id.clone(), new_pwd.clone());

                let new_mail = match message_object.get("mail") {
                    Some(m) => {
                        println!("new_mail some: \"{}\"", &m.as_string().unwrap());
                        let re = Regex::new(r"(?i)\A[\w-\.]+@[a-z\d]+\.[a-z]+\z").unwrap();

                        match re.is_match(&(m.as_string().unwrap().to_string())) {
                            false => {
                                println!("mail mismatch: {}", &m);
                                return Err(rustless::ErrorResponse{
                                    error: Box::new(InvalidMail) as Box<RError + Send>,
                                    response: None
                                })
                            },
                            true  => {
                                println!("mail match: {}", &m);
                                Some(m.as_string().unwrap().to_string())
                            }
                        }
                    },
                    None   => {
                        None
                    }
                };

                let new_user = User {
                                name: new_name.clone(),
                                id: new_id,
                                mail: new_mail.clone(),
                                signature: new_sig,
                                created: new_created
                              };
                println!("Post user: new User: {}", &new_user);
                let users = storage_clone.lock().unwrap().users.clone();

                let user = get_user_by_name(users, &new_name);

                match user {
                    Some(_)  => {
                        client.set_status(status::StatusCode::Conflict);
                        client.text(format!("User with name {} exists already.", &new_name).to_string()) //sic
                    },
                    None    => {
                        let test = &new_user.to_json();
                        // let test2 = json::Json::from_str(&test).unwrap();
                        storage_clone.lock().unwrap().users.push(new_user);
                        save_gamekey(storage_clone.lock().unwrap().clone());
                        client.json(&test)
                    }
                }


            })
        });

        let storage_clone = storage.clone();
        api.get("user/:id", |endpoint| {
            endpoint.summary("Retrieves user data");
            endpoint.desc("Use this to retrieve a users data");
            endpoint.params(|params| {
                params.req_typed("id", json_dsl::string());
                params.req_typed("pwd", json_dsl::string());
                params.opt_typed("byname", json_dsl::boolean());
            });
            endpoint.handle(move |mut client, params| {
                let message_object = params.as_object().unwrap();


                // TODO: Hahaha, fix this shit
                let id = String::from_utf8(url::percent_encoding::percent_decode( (message_object.get("id").unwrap().as_string().unwrap().to_string().as_bytes()))).unwrap();
                let pwd  = message_object.get("pwd").unwrap().as_string().unwrap().to_string();
                let byname: bool = match message_object.get("byname") {
                    Some(v) => {
                        v.as_boolean().unwrap()
                    },
                    None => {
                        false
                    }
                };

                // let test = url::percent_encoding::percent_decode( (id.as_bytes()) );
                //
                // println!("\n\n\n\ntest: {}\n\n", String::from_utf8(test).unwrap() );

                let users = storage_clone.lock().unwrap().users.clone();

                let user = match byname {
                    true  => {
                        get_user_by_name(users, &id)
                    },
                    false => {
                        get_user_by_id(users, &id)
                    }
                };

                match user {
                    Some(e) => {
                        if e.signature == auth_signature(e.id.clone(), pwd.clone()) {
                            let user_json = &e.to_json();

                            client.json(&user_json)
                        } else {
                            client.set_status(status::StatusCode::Unauthorized);
                            client.text("unauthorized, please provide correct credentials".to_string())
                        }
                    },
                    None    => {
                        client.set_status(status::StatusCode::NotFound);
                        client.text("User not found".to_string())
                    }
                }

            })
        });

        let storage_clone = storage.clone();
        api.put("user/:id", |endpoint| {
            endpoint.summary("Updates a user");
            endpoint.desc("Use this to update a users data");
            endpoint.params(|params| {
                params.req_typed("id", json_dsl::string());
                params.req_typed("pwd", json_dsl::string());
                params.opt_typed("name", json_dsl::string());
                params.opt_typed("mail", json_dsl::string());
                params.opt_typed("newpwd", json_dsl::string());
            });
            endpoint.handle(move |client, params| {
                let message_object = params.as_object().unwrap();

                let id = message_object.get("id").unwrap().as_string().unwrap().to_string();
                let pwd  = message_object.get("pwd").unwrap().as_string().unwrap().to_string();

                let users = storage_clone.lock().unwrap().users.clone();
                let user = get_user_by_id(users, &id);

                let mut user_unwrapped = match user {
                    Some(e) => {
                        if e.signature != auth_signature(e.id.clone(), pwd.clone()) {
                            println!("signature mismatch: {}", &e);
                            return Err(rustless::ErrorResponse{
                                error: Box::new(UnauthorizedError) as Box<RError + Send>,
                                response: None
                            })
                        } else {
                            e
                        }
                    },
                    None   => {
                        println!("user not found: {}", &id);
                        return Err(rustless::ErrorResponse{
                            error: Box::new(NotFoundError) as Box<RError + Send>,
                            response: None
                        })
                    }
                };

                let usr = user_unwrapped.clone();
                let name: String = match message_object.get("name") {
                    Some(v) => {
                        v.as_string().unwrap().to_string()
                    },
                    None => {
                        usr.name
                    }
                };

                let usr = user_unwrapped.clone();
                let mail: Option<String> = match message_object.get("mail") {
                    Some(v) => {
                        Some(v.as_string().unwrap().to_string())
                    },
                    None => {
                        usr.mail
                    }
                };

                let usr = user_unwrapped.clone();
                let newsig: String = match message_object.get("newpwd") {
                    Some(v) => {
                        auth_signature(usr.id, v.as_string().unwrap().to_string())
                    },
                    None => {
                        usr.signature
                    }
                };

                user_unwrapped.name = name;
                user_unwrapped.mail = mail;
                user_unwrapped.signature = newsig;

                println!("Update User ID");
                let test = &user_unwrapped.to_json();
                save_gamekey(storage_clone.lock().unwrap().clone());
                client.json(&test)
            })
        });


        let storage_clone = storage.clone();
        api.delete("user/:id", |endpoint| {
            endpoint.summary("Deletes a user");
            endpoint.desc("Use this to delete a user");
            endpoint.params(|params| {
                params.req_typed("id", json_dsl::string());
                params.req_typed("pwd", json_dsl::string());
            });
            endpoint.handle(move |client, params| {
                let message_object = params.as_object().unwrap();

                let id = message_object.get("id").unwrap().as_string().unwrap().to_string();
                let pwd  = message_object.get("pwd").unwrap().as_string().unwrap().to_string();

                let users = storage_clone.lock().unwrap().users.clone();
                let user = get_user_by_id(users, &id);

                match user {
                    Some(e) => {
                        if e.signature != auth_signature(e.id.clone(), pwd.clone()) {
                            println!("signature mismatch: {}", &e);
                            return Err(rustless::ErrorResponse{
                                error: Box::new(UnauthorizedError) as Box<RError + Send>,
                                response: None
                            })
                        }
                    },
                    None   => {
                        println!("user not found: {}", &id);
                        return Err(rustless::ErrorResponse{
                            error: Box::new(NotFoundError) as Box<RError + Send>,
                            response: None
                        })
                    }
                };

                let ref mut usrs = storage_clone.lock().unwrap().users;

                usrs.iter()
                .position(|ref n| n.id == id)
                .map(|e| usrs.remove(e));

                client.text("User removed".to_string())
            })
        });

        api.get("games", |endpoint| {
            endpoint.summary("Lists all registered games");
            endpoint.desc("Use this to list all registered games");
            endpoint.handle(|client, params| {
                println!("Update User ID");
                client.json(params)
            })
        });

        api.post("game", |endpoint| {
            endpoint.summary("Creates a game");
            endpoint.desc("Use this to create a game");
            endpoint.params(|params| {
                params.req_typed("name", json_dsl::string());
                params.req_typed("secret", json_dsl::string());
                params.opt_typed("url", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                println!("Create game");
                client.json(params)
            })
        });

        api.get("game/:id", |endpoint| {
            endpoint.summary("Creates a game");
            endpoint.desc("Use this to create a game");
            endpoint.params(|params| {
                params.req_typed("secret", json_dsl::string());
                params.req_typed("id", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                println!("Get game");
                client.json(params)
            })
        });

        api.put("game/:id", |endpoint| {
            endpoint.summary("Updates a game");
            endpoint.desc("Use this to update a game");
            endpoint.params(|params| {
                params.req_typed("id", json_dsl::string());
                params.req_typed("secret", json_dsl::string());
                params.opt_typed("name", json_dsl::string());
                params.opt_typed("url", json_dsl::string());
                params.opt_typed("newsecret", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                println!("Update game");
                client.json(params)
            })
        });

        api.delete("game/:id", |endpoint| {
            endpoint.summary("Delete a game");
            endpoint.desc("Use this to delete a game");
            endpoint.params(|params| {
                params.req_typed("id", json_dsl::string());
                params.req_typed("secret", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                println!("Delete game");
                client.json(params)
            })
        });

        api.get("gamestate/:gameid", |endpoint| {
            endpoint.summary("Retrieves all gamestates for a game");
            endpoint.desc("..");
            endpoint.params(|params| {
                params.req_typed("gameid", json_dsl::string());
                params.req_typed("secret", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                println!("Get gamestates");
                client.json(params)
            })
        });

        api.get("gamestate/:gameid/:userid", |endpoint| {
            endpoint.summary("Retrieves gamestates for a game and user");
            endpoint.desc("..");
            endpoint.params(|params| {
                params.req_typed("gameid", json_dsl::string());
                params.req_typed("userid", json_dsl::string());
                params.req_typed("secret", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                println!("Get game and user");
                client.json(params)
            })
        });

        api.post("gamestate/:gameid/:userid", |endpoint| {
            endpoint.summary("Updates gamestates for a game and user");
            endpoint.desc("..");
            endpoint.params(|params| {
                params.req_typed("gameid", json_dsl::string());
                params.req_typed("userid", json_dsl::string());
                params.req_typed("secret", json_dsl::string());
                params.req_typed("state", json_dsl::string());
                //TODO check state thingy
            });
            endpoint.handle(|client, params| {
                println!("Get game and user");
                client.json(params)
            })
        });

    }));

    let chain = iron::Chain::new(app);

    iron::Iron::new(chain).http("0.0.0.0:4000").unwrap();
    println!("On 4000");

}
