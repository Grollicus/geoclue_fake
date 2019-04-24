extern crate dbus;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate lazy_static;

use dbus::{Connection, tree};
use std::sync::{Arc, Mutex};
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

#[derive(serde::Deserialize, Default, Clone, Copy, Debug)]
struct LocationData {
    latitude: f64,
    longitude: f64,
    accuracy: f64,
    altitude: f64,
    speed: f64,
    heading: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct Client {
    desktop_id: String,
    distance_threshold: u32,
}

#[derive(Copy, Clone, Default, Debug)]
struct TData;
impl tree::DataType for TData {
    type Tree = ();
    type ObjectPath = Option<usize>;
    type Property = ();
    type Interface = ();
    type Method = ();
    type Signal = ();
}

fn create_client(clientno: usize, tree: &mut dbus::tree::Tree<dbus::tree::MTFnMut<TData>, TData>, conn: &mut Connection) {
    let f = tree::Factory::new_fnmut::<TData>();

    let location_updated = Arc::new(f.signal("LocationUpdated", ()).sarg::<dbus::Path,_>("old").sarg::<dbus::Path,_>("new"));
    let location_updated_clone = location_updated.clone();

    let path = f.object_path(format!("/org/freedesktop/GeoClue2/Client/{}", clientno), Some(clientno)).introspectable().add(
        f.interface("org.freedesktop.GeoClue2.Client", ())
            .add_m(f.method("Start", (), move |m| {
                let clientno = m.path.get_data().unwrap();
                let sig = location_updated.msg(m.path.get_name(), m.iface.get_name()).append2::<dbus::Path, dbus::Path>(
                    dbus::Path::new("/").unwrap(),
                    dbus::Path::new(format!("/org/freedesktop/GeoClue2/Client/{}/Location/0", clientno)).unwrap()
                );
                return Ok(vec!(m.msg.method_return(), sig));
            }))
            .add_m(f.method("Stop", (), |m| { Ok(vec!(m.msg.method_return())) }))
            .add_s(location_updated_clone)
            .add_p(f.property::<&str, _>("DesktopId", ()).access(tree::Access::ReadWrite)
                .on_set(|i, m| {
                    let clientno = m.path.get_data().unwrap();
                    let new_desktop_id = i.read::<&str>()?;
                    {
                        let mut clients = CLIENTS.lock().unwrap();
                        let client: &mut Client = clients.get_mut(clientno).unwrap();
                        client.desktop_id.truncate(0);
                        client.desktop_id.push_str(new_desktop_id);
                    }
                    Ok(())
                }).on_get(|i, m| {
                    let clientno = m.path.get_data().unwrap();
                    i.append(CLIENTS.lock().unwrap()[clientno].desktop_id.as_str());
                    Ok(())
            }))
            .add_p(f.property::<u32, _>("DistanceThreshold", ()).access(tree::Access::ReadWrite)
                .on_set(|i, m| {
                    let clientno = m.path.get_data().unwrap();
                    let new_distance_threshold = i.read::<u32>()?;
                    {
                        let mut clients = CLIENTS.lock().unwrap();
                        let client: &mut Client = clients.get_mut(clientno).unwrap();
                        client.distance_threshold = new_distance_threshold;
                    }
                    Ok(())
                }).on_get(|i, m| {
                    let clientno = m.path.get_data().unwrap();
                    i.append(CLIENTS.lock().unwrap()[clientno].distance_threshold);
                    Ok(())
            }))
        );
    tree.insert(path);
    conn.register_object_path(format!("/org/freedesktop/GeoClue2/Client/{}", clientno).as_str()).unwrap();
}

fn create_location(clientno: usize, tree: &mut dbus::tree::Tree<dbus::tree::MTFnMut<TData>, TData>, conn: &mut Connection) {
    let f = tree::Factory::new_fnmut::<TData>();

    let path = f.object_path(format!("/org/freedesktop/GeoClue2/Client/{}/Location/0", clientno), Some(clientno)).introspectable().add(
        f.interface("org.freedesktop.GeoClue2.Location", ())
            .add_p(f.property::<f64, _>("Latitude", ()).access(tree::Access::Read).on_get(move |i, m| {
                i.append(LOCATION_DATA.lock().unwrap().latitude);
                let clientno = m.path.get_data().unwrap();
                println!("Latitude queried by #{}: '{}'", clientno, CLIENTS.lock().unwrap()[clientno].desktop_id.as_str());
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Longitude", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(LOCATION_DATA.lock().unwrap().longitude);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Accuracy", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(LOCATION_DATA.lock().unwrap().accuracy);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Altitude", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(LOCATION_DATA.lock().unwrap().altitude);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Speed", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(LOCATION_DATA.lock().unwrap().speed);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Heading", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(LOCATION_DATA.lock().unwrap().heading);
                Ok(())
            }))
            .add_p(f.property::<(u64, u64), _>("Timestamp", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append::<(u64, u64)>((LOCATION_DATA.lock().unwrap().timestamp, 0));
                Ok(())
            }))
    );

    tree.insert(path);
    conn.register_object_path(format!("/org/freedesktop/GeoClue2/Client/{}/Location/0", clientno).as_str()).unwrap();
}

fn add_client(clients: &mut Vec<Client>, mut tree: &mut dbus::tree::Tree<dbus::tree::MTFnMut<TData>, TData>, mut conn: &mut Connection) -> usize {
    let clientno = clients.len();

    clients.push(Client{desktop_id: String::new(), distance_threshold: 10000});
    create_client(clientno, &mut tree, &mut conn);
    create_location(clientno, &mut tree, &mut conn);

    return clientno
}


// TODO make CLIENTS a dictionary
lazy_static! {
    static ref LOCATION_DATA : Mutex<LocationData> = {Mutex::new(LocationData::default())};
    static ref CLIENTS: Mutex<Vec<Client>> = {Mutex::new(vec![])};
}

fn main() -> Result<(), Error> {
    let argv: Vec<_> = std::env::args_os().collect();

    if let Ok(mut f) = std::fs::File::open(argv.get(1).unwrap_or(&std::ffi::OsString::from(CONFIG_DEFAULT_PATH))) {
        let mut file_contents = String::new();
        f.read_to_string(&mut file_contents)?;
        *LOCATION_DATA.lock().unwrap() = toml::from_str(file_contents.as_str())?;
        println!("Loaded config file");
    }
    let mut c = Connection::get_private(dbus::BusType::System)?;

    let f = tree::Factory::new_fnmut::<(TData)>();
    let mut t = f.tree(()).add(f.object_path("/org/freedesktop/GeoClue2/Manager", None).introspectable().add(
        f.interface("org.freedesktop.GeoClue2.Manager", ())
            .add_m(f.method("GetClient", (), |m| {
                // add_client(&mut CLIENTS.lock().unwrap(), &mut t, &mut c); TODO
                Ok(vec!(m.msg.method_return().append(dbus::Path::new("/org/freedesktop/GeoClue2/Client/0").unwrap())))
            }).outarg::<dbus::Path, _>("client"))
            .add_m(f.method("CreateClient", (), |m| {
                Ok(vec!(m.msg.method_return().append(dbus::Path::new("/org/freedesktop/GeoClue2/Client/0").unwrap())))
            }).outarg::<dbus::Path, _>("client"))
        )
    );

    c.register_name("org.freedesktop.GeoClue2", dbus::NameFlag::ReplaceExisting as u32)?;
    t.set_registered(&c, true)?;

    add_client(&mut CLIENTS.lock().unwrap(), &mut t, &mut c);
    loop {
        for msg in c.incoming(1_000_000) {
            if let Some(reponses) = t.handle(&msg) {
                for resp in reponses {
                    if c.send(resp).is_err() {
                        println!("Dbus send error");
                    }
                }
            }
        }
    }
}
