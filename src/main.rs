/*
    geoclue_fake - a fake GeoClue2 dbus service
    Copyright (C) 2019  Grollicus

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

extern crate dbus;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate lazy_static;

use dbus::strings::Path;
use dbus::blocking::Connection;
use dbus_crossroads::{Context, Crossroads, IfaceToken, MethodErr};
use std::sync::Mutex;
use std::io::Read;


const CONFIG_DEFAULT_PATH: &str = "/etc/geoclue_fake.toml";


#[derive(Debug)]
enum Error {
    DbusError(dbus::Error),
    IoError(std::io::Error),
    ConfigError(toml::de::Error),
}

impl From<dbus::Error> for Error {
    fn from(e: dbus::Error) -> Error {
        Error::DbusError(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}
impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Error {
        Error::ConfigError(e)
    }
}

#[derive(serde::Deserialize, Default, Clone, Debug)]
struct LocationData {
    latitude: f64,
    longitude: f64,
    accuracy: f64,
    altitude: f64,
    speed: f64,
    heading: f64,
    #[serde(default="String::new")]
    description: String,
    timestamp: u64,
}

// TODO make CLIENTS a dictionary
lazy_static! {
    static ref LOCATION_DATA : Mutex<LocationData> = Mutex::new(LocationData::default());
}

struct ManagerState {
    next_id: u32
}

#[derive(Default)]
struct ClientState {
    client_id: u32,
    sender: String,
    distance_threshold: u32,
    time_threshold: u32, // not supported
    desktop_id: String,
    requested_accuracy_level: u32, // not supported
    active: bool,
}

fn lookup_client_state<'l>(cr: &'l mut Crossroads, path: &Path<'l>) -> Result<&'l mut ClientState, MethodErr> {
    let path_suffix = path.strip_prefix("/org/freedesktop/GeoClue2/Client/").ok_or_else(|| MethodErr::failed(""))?;
    let mut suffix_iterator = path_suffix.split_terminator("/");
    let client_id = suffix_iterator.next().unwrap();
    if suffix_iterator.next().is_none() {
        return Err(MethodErr::failed(""));
    }
    let client_id: u32 = client_id.parse().map_err(|_| MethodErr::failed(""))?;
    let client_path = format!("/org/freedesktop/GeoClue2/Client/{}", client_id);
    return cr.data_mut(&client_path.into()).ok_or_else(|| MethodErr::failed(""));
}

fn create_location(cr: &mut Crossroads) -> IfaceToken<()> {
    cr.register("org.freedesktop.GeoClue2.Location", |b| {
        b.property("Latitude")
            .get_with_cr(|ctx, cr: &mut Crossroads| {
                let client_state = lookup_client_state(cr, ctx.path())?;
                println!("Latitude queried by #{}: '{}'", client_state.client_id, client_state.desktop_id);
                LOCATION_DATA.lock().map_or_else(|_| Err(MethodErr::failed("Could not get Location data")), |l| Ok(l.latitude))
            });
        b.property("Longitude").get(|_ctx, _| LOCATION_DATA.lock().map_or_else(|_| Err(MethodErr::failed("Could not get Location data")), |l| Ok(l.longitude)));
        b.property("Accuracy").get(|_ctx, _| LOCATION_DATA.lock().map_or_else(|_| Err(MethodErr::failed("Could not get Location data")), |l| Ok(l.accuracy)));
        b.property("Altitude").get(|_ctx, _| LOCATION_DATA.lock().map_or_else(|_| Err(MethodErr::failed("Could not get Location data")), |l| Ok(l.altitude)));
        b.property("Speed").get(|_ctx, _| LOCATION_DATA.lock().map_or_else(|_| Err(MethodErr::failed("Could not get Location data")), |l| Ok(l.speed)));
        b.property("Heading").get(|_ctx, _| LOCATION_DATA.lock().map_or_else(|_| Err(MethodErr::failed("Could not get Location data")), |l| Ok(l.heading)));
        b.property("Description").get(|_ctx, _| LOCATION_DATA.lock().map_or_else(|_| Err(MethodErr::failed("Could not get Location data")), |l| Ok(l.description.clone())));
        b.property("Timestamp").get(|_ctx, _| LOCATION_DATA.lock().map_or_else(|_| Err(MethodErr::failed("Could not get Location data")), |l| Ok(l.timestamp)));
    })
}

fn create_client_token(cr: &mut Crossroads) -> IfaceToken<ClientState> {
    cr.register("org.freedesktop.GeoClue2.Client", |b| {
        b.property("Location").get(|_ctx, client_state: &mut ClientState| {
            if client_state.active {
                Ok(Path::from(format!("/org/freedesktop/GeoClue2/Client/{}/Location/0", client_state.client_id)))
            } else {
                Ok(Path::from("/"))
            }
        });
        b.property("DistanceThreshold")
            .get(|_ctx, client_state: &mut ClientState| Ok(client_state.distance_threshold.clone()))
            .set(|_ctx, client_state, new_value| {
                client_state.distance_threshold = new_value.clone();
                Ok(Some(new_value))
            });
        b.property("TimeThreshold")
            .get(|_ctx, client_state: &mut ClientState| Ok(client_state.time_threshold.clone()))
            .set(|_ctx, client_state, new_value| {
                client_state.time_threshold = new_value.clone();
                Ok(Some(new_value))
            });
        b.property("DesktopId")
            .get(|_ctx, client_state: &mut ClientState| Ok(client_state.desktop_id.clone()))
            .set(|_ctx, client_state, new_value| {
                client_state.desktop_id = new_value.clone();
                Ok(Some(new_value))
            });
        b.property("RequestedAccuracyLevel")
            .get(|_ctx, client_state: &mut ClientState| Ok(client_state.requested_accuracy_level.clone()))
            .set(|_ctx, client_state, new_value| {
                client_state.requested_accuracy_level = new_value.clone();
                Ok(Some(new_value))
            });
        b.property("Active").get(|_ctx, client_state: &mut ClientState| Ok(client_state.active.clone()));

        b.signal::<(Path,  Path), _>("LocationUpdated", ("old", "new"));
        b.method_with_cr("Start", (), (), |ctx: &mut Context, cr: &mut Crossroads, _: ()| {
            let _sender = ctx.message().sender().ok_or_else(|| MethodErr::failed("Unknown Sender"))?;
            let client_state: &mut ClientState = cr.data_mut(ctx.path()).ok_or_else(|| MethodErr::no_path(ctx.path()))?;
            let client_id = client_state.client_id;
            println!("Start called by Client {}", client_id);

            let location_path = format!("/org/freedesktop/GeoClue2/Client/{}/Location/0", client_id);
            client_state.active = true;
            let location_token = create_location(cr);
            cr.insert(location_path.clone(), &[location_token], ());

            let location_path = Path::from(format!("/org/freedesktop/GeoClue2/Client/{}/Location/0", client_id));
            let sig = ctx.make_signal("LocationUpdated", (Path::from("/"), location_path));
            ctx.push_msg(sig);
            Ok(())
        });
        b.method_with_cr("Stop", (), (), |ctx: &mut Context, cr: &mut Crossroads, _: ()| {
            let _sender = ctx.message().sender().ok_or_else(|| MethodErr::failed("Unknown Sender"))?;
            let client_state: &mut ClientState = cr.data_mut(ctx.path()).ok_or_else(|| MethodErr::no_path(ctx.path()))?;
            println!("Stop called by Client {}", client_state.client_id);
            client_state.active = false;
            Ok(())
        });
    })
}

fn main() -> Result<(), Error> {
    let argv: Vec<_> = std::env::args_os().collect();

    if let Ok(mut f) = std::fs::File::open(argv.get(1).unwrap_or(&std::ffi::OsString::from(CONFIG_DEFAULT_PATH))) {
        let mut file_contents = String::new();
        f.read_to_string(&mut file_contents)?;
        *LOCATION_DATA.lock().unwrap() = toml::from_str(file_contents.as_str())?;
        println!("Loaded config file");
    }

    let c  = if cfg!(debug_assertions) {
        Connection::new_session().expect("Could not acquire Session D-Bus Connection for debugging")
    } else {
        Connection::new_system().expect("Could not acquire System D-Bus Connection")
    };
    c.request_name("org.freedesktop.GeoClue2", true, true, true).expect("Requesting name org.freedesktop.GeoClue2 failed");

    let mut cr = Crossroads::new();

    let manager_token = cr.register("org.freedesktop.GeoClue2.Manager", |b| {
        b.method_with_cr("GetClient", (), ("ClientID",), |ctx: &mut Context, cr: &mut Crossroads, _: ()| {
            let sender = ctx.message().sender().ok_or_else(|| MethodErr::failed("Unknown Sender"))?;
            let manager_state: &mut ManagerState = cr.data_mut(ctx.path()).ok_or_else(|| MethodErr::no_path(ctx.path()))?;
            println!("Client ID requested by {}", &sender);

            let this_id = manager_state.next_id;
            manager_state.next_id = manager_state.next_id.checked_add(1).expect("Overflowed an u32 requesting client ids. Congratulations, you've earned this crash!");
            let client_path = format!("/org/freedesktop/GeoClue2/Client/{}", this_id);

            let client_token = create_client_token(cr);
            cr.insert(client_path.clone(), &[client_token], ClientState { client_id: this_id, sender: sender.to_string(), ..ClientState::default() });

            Ok((Path::from(client_path),))
        });
    });

    cr.insert("/org/freedesktop/GeoClue2/Manager", &[manager_token], ManagerState { next_id: 0 });
    cr.serve(&c).expect("dbus serve failed");
    unreachable!()

}
