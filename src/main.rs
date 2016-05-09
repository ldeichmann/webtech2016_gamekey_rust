#[macro_use]
extern crate rustless;

extern crate iron;
extern crate url;
extern crate rustc_serialize;
extern crate valico;
extern crate chrono;
extern crate uuid;

use std::collections::LinkedList;
use std::sync::{Arc, Mutex};
use rustc_serialize::base64::{STANDARD, ToBase64};
use uuid::Uuid;

use std::fmt;
use std::error;
use std::error::Error as StdError;
use valico::json_dsl;

use rustless::server::status;
use std::error::Error;
use rustless::batteries::swagger;
use rustless::{Nesting};
use rustc_serialize::json;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;


#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct User {
    name: String,
    id: String,
    email: String,
    created: String,
    signature : String
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {} {}", self.name, self.id, self.email, self.signature, self.created)
    }
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Game {
    name: String,
    id: String,
    url: String,
    signature: String,
    created: String
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {} {}", self.name, self.id, self.url, self.signature, self.created)
    }
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Gamestate {
    gameid: String,
    userid: String,
    created: String,
    state: String
}

impl fmt::Display for Gamestate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.gameid, self.userid, self.created, self.state)
    }
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct GameKey {
    users: LinkedList<User>,
    games: LinkedList<Game>,
    gamestates: LinkedList<Gamestate>
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

fn create_gamekey() -> GameKey {

    let users: LinkedList<User> = LinkedList::new();
    let games: LinkedList<Game> = LinkedList::new();
    let gamestates: LinkedList<Gamestate> = LinkedList::new();

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

    return create_gamekey();

}

fn save_gamekey(gk: GameKey) {
    let js = json::encode(&gk).unwrap();
    println!("\nsave_gamekey got:\n{}\n", (&js).to_string());

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


fn main() {

    let storage = Arc::new(Mutex::new(get_gamekey()));
    // let users = Arc::new(Mutex::new(LinkedList::new()));
    // let mut games = LinkedList::new();
    // let mut gamestates = LinkedList::new();


    let app = rustless::Application::new(rustless::Api::build(|api| {
        // api.prefix("api");
        // api.version("v1", rustless::Versioning::AcceptHeader("chat"));

        api.mount(swagger::create_api("api-docs"));

        api.error_formatter(|err, _media| {
            match err.downcast::<InvalidMail>() {
                Some(_) => {
                    return Some(rustless::Response::from(
                        status::StatusCode::BadRequest,
                        Box::new("Please provide correct `token` parameter")
                    ))
                },
                None => None
            }
        });

        let storage_clone = storage.clone();
        api.get("users", |endpoint| {
            endpoint.summary("Lists all registered users");
            endpoint.desc("");
            // let storage_clone = storage.clone();
            endpoint.handle(move |client, _| {
                let users: LinkedList<User> = storage_clone.lock().unwrap().users.clone();

                let user_json = match json::encode(&users) {
                    Ok(v) => {
                        v
                    },
                    Err(err) => {
                        panic!("fuck {:?}", err);
                    }
                };

                println!("get user: {:?}", user_json.to_string());

                let test = json::Json::from_str(&user_json.to_string()).unwrap();

                client.json(&test)
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
            endpoint.handle(move |client, params| {
                let message_object = params.as_object().unwrap();

                let new_name = message_object.get("name").unwrap().as_string().unwrap().to_string();
                let new_pwd  = message_object.get("pwd").unwrap().as_string().unwrap().to_string();
                let new_sig = (String::new() + &new_name + &new_pwd).as_bytes().to_base64(STANDARD);
                let new_mail = message_object.get("mail").unwrap().as_string().unwrap().to_string();
                let new_created = chrono::UTC::now().to_string();
                let new_id = Uuid::new_v4().to_string();

                let new_user = User {
                                name: new_name,
                                id: new_id,
                                email: new_mail,
                                signature: new_sig,
                                created: new_created
                              };
                println!("new_user: {}", &new_user);
                storage_clone.lock().unwrap().users.push_front(new_user);
                save_gamekey(storage_clone.lock().unwrap().clone());
                client.json(params)
            })
        });

        api.get("user/:id", |endpoint| {
            endpoint.summary("Retrieves user data");
            endpoint.desc("Use this to retrieve a users data");
            endpoint.params(|params| {
                params.req_typed("id", json_dsl::string());
                params.req_typed("pwd", json_dsl::string());
                params.opt_typed("byname", json_dsl::boolean());
            });
            endpoint.handle(|client, params| {
                let id = params.find_path(&["id"]).unwrap().to_string();
                let pwd  = params.find_path(&["pwd"]).unwrap().to_string();
                match params.find_path(&["byname"]) {
                    None => { println!("no byname"); }
                    Some(byname) => {
                        println!("{} {} {}", id, pwd, byname);
                    }
                }
                println!("{} {}", id, pwd);
                client.json(params)
            })
        });

        api.put("user/:id", |endpoint| {
            endpoint.summary("Updates a user");
            endpoint.desc("Use this to update a users data");
            endpoint.params(|params| {
                params.req_typed("name", json_dsl::string());
                params.req_typed("pwd", json_dsl::string());
                params.opt_typed("name", json_dsl::string());
                params.opt_typed("mail", json_dsl::string());
                params.opt_typed("newpwd", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                println!("Update User ID");
                client.json(params)
            })
        });

        api.delete("user/:id", |endpoint| {
            endpoint.summary("Deletes a user");
            endpoint.desc("Use this to delete a user");
            endpoint.params(|params| {
                params.req_typed("id", json_dsl::string());
                params.req_typed("pwd", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                println!("Delete User ID");
                client.json(params)
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
