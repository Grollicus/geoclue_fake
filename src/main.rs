extern crate dbus;
extern crate serde;
extern crate toml;

use dbus::{Connection, tree};
use std::sync::Arc;
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

#[derive(serde::Deserialize, Default, Clone, Copy, serde::Serialize)]
struct LocationData {
    latitude: f64,
    longitude: f64,
    accuracy: f64,
    altitude: f64,
    speed: f64,
    heading: f64,
    timestamp: u64,
}

fn main() -> Result<(), Error> {
    let argv: Vec<_> = std::env::args_os().collect();

    let mut location_data: LocationData = LocationData::default();
    if let Ok(mut f) = std::fs::File::open(argv.get(1).unwrap_or(&std::ffi::OsString::from(CONFIG_DEFAULT_PATH))) {
        let mut file_contents = String::new();
        f.read_to_string(&mut file_contents)?;
        location_data = toml::from_str(file_contents.as_str())?;
    }

    let f = tree::Factory::new_fn::<()>();
    let location_updated = Arc::new(f.signal("LocationUpdated", ()).sarg::<dbus::Path,_>("old").sarg::<dbus::Path,_>("new"));
    let location_updated_clone = location_updated.clone();

    let t = f.tree(()).add(f.object_path("/org/freedesktop/GeoClue2/Manager", ()).introspectable().add(
        f.interface("org.freedesktop.GeoClue2.Manager", ())
            .add_m(f.method("GetClient", (), |m| {
                Ok(vec!(m.msg.method_return().append(dbus::Path::new("/org/freedesktop/GeoClue2/Client/1").unwrap())))
            }).outarg::<dbus::Path, _>("client"))
            .add_m(f.method("CreateClient", (), |m| {
                Ok(vec!(m.msg.method_return().append(dbus::Path::new("/org/freedesktop/GeoClue2/Client/1").unwrap())))
            }).outarg::<dbus::Path, _>("client"))
        )
    ).add(f.object_path("/org/freedesktop/GeoClue2/Client/1", ()).introspectable().add(
        f.interface("org.freedesktop.GeoClue2.Client", ())
            .add_m(f.method("Start", (), move |m| {
                let sig = location_updated.msg(m.path.get_name(), m.iface.get_name()).append2::<dbus::Path, dbus::Path>(
                    dbus::Path::new("/").unwrap(),
                    dbus::Path::new("/org/freedesktop/GeoClue2/Client/1/Location/0").unwrap()
                );
                return Ok(vec!(m.msg.method_return(), sig));
            }))
            .add_m(f.method("Stop", (), |m| { Ok(vec!(m.msg.method_return())) }))
            .add_s(location_updated_clone)
            .add_p(f.property::<&str, _>("DesktopId", ()).access(tree::Access::Write).on_set(|i, _m| {
                println!("DesktopId set to {}", i.read::<&str>()?);
                Ok(())
            }))
            .add_p(f.property::<u32, _>("DistanceThreshold", ()).access(tree::Access::Write).on_set(|i, _m| {
                println!("DistanceThreshold set to {}", i.read::<u32>()?);
                Ok(())
            }))
        )
    ).add(f.object_path("/org/freedesktop/GeoClue2/Client/1/Location/0", ()).introspectable().add(
        f.interface("org.freedesktop.GeoClue2.Location", ())
            .add_p(f.property::<f64, _>("Latitude", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(location_data.latitude);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Longitude", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(location_data.longitude);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Accuracy", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(location_data.accuracy);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Altitude", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(location_data.altitude);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Speed", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(location_data.speed);
                Ok(())
            }))
            .add_p(f.property::<f64, _>("Heading", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append(location_data.heading);
                Ok(())
            }))
            .add_p(f.property::<(u64, u64), _>("Timestamp", ()).access(tree::Access::Read).on_get(move |i, _m| {
                i.append::<(u64, u64)>((location_data.timestamp, 0));
                Ok(())
            }))
    ));

    let c = Connection::get_private(dbus::BusType::System)?;
    c.register_name("org.freedesktop.GeoClue2", dbus::NameFlag::ReplaceExisting as u32)?;
    t.set_registered(&c, true)?;
    c.add_handler(t);
    loop { c.incoming(1_000_000).next(); }
}
