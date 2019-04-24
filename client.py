#!/usr/bin/env python3

# import sys
from pydbus import Variant, SessionBus  # , SystemBus
from gi.repository import GLib


# manager_desc = """<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
#                       "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
# <!-- GDBus 2.60.0 -->
# <node>
#   <interface name="org.freedesktop.DBus.Properties">
#     <method name="Get">
#       <arg type="s" name="interface_name" direction="in"/>
#       <arg type="s" name="property_name" direction="in"/>
#       <arg type="v" name="value" direction="out"/>
#     </method>
#     <method name="GetAll">
#       <arg type="s" name="interface_name" direction="in"/>
#       <arg type="a{sv}" name="properties" direction="out"/>
#     </method>
#     <method name="Set">
#       <arg type="s" name="interface_name" direction="in"/>
#       <arg type="s" name="property_name" direction="in"/>
#       <arg type="v" name="value" direction="in"/>
#     </method>
#     <signal name="PropertiesChanged">
#       <arg type="s" name="interface_name"/>
#       <arg type="a{sv}" name="changed_properties"/>
#       <arg type="as" name="invalidated_properties"/>
#     </signal>
#   </interface>
#   <interface name="org.freedesktop.DBus.Introspectable">
#     <method name="Introspect">
#       <arg type="s" name="xml_data" direction="out"/>
#     </method>
#   </interface>
#   <interface name="org.freedesktop.DBus.Peer">
#     <method name="Ping"/>
#     <method name="GetMachineId">
#       <arg type="s" name="machine_uuid" direction="out"/>
#     </method>
#   </interface>
#   <interface name="org.freedesktop.GeoClue2.Manager">
#     <method name="GetClient">
#       <arg type="o" name="client" direction="out"/>
#     </method>
#     <method name="CreateClient">
#       <arg type="o" name="client" direction="out"/>
#     </method>
#     <method name="DeleteClient">
#       <arg type="o" name="client" direction="in"/>
#     </method>
#     <method name="AddAgent">
#       <arg type="s" name="id" direction="in"/>
#     </method>
#     <property type="b" name="InUse" access="read"/>
#     <property type="u" name="AvailableAccuracyLevel" access="read"/>
#   </interface>
# </node>"""
#
# <!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
#                       "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
# <!-- GDBus 2.60.0 -->
# <node>
#   <interface name="org.freedesktop.DBus.Properties">
#     <method name="Get">
#       <arg type="s" name="interface_name" direction="in"/>
#       <arg type="s" name="property_name" direction="in"/>
#       <arg type="v" name="value" direction="out"/>
#     </method>
#     <method name="GetAll">
#       <arg type="s" name="interface_name" direction="in"/>
#       <arg type="a{sv}" name="properties" direction="out"/>
#     </method>
#     <method name="Set">
#       <arg type="s" name="interface_name" direction="in"/>
#       <arg type="s" name="property_name" direction="in"/>
#       <arg type="v" name="value" direction="in"/>
#     </method>
#     <signal name="PropertiesChanged">
#       <arg type="s" name="interface_name"/>
#       <arg type="a{sv}" name="changed_properties"/>
#       <arg type="as" name="invalidated_properties"/>
#     </signal>
#   </interface>
#   <interface name="org.freedesktop.DBus.Introspectable">
#     <method name="Introspect">
#       <arg type="s" name="xml_data" direction="out"/>
#     </method>
#   </interface>
#   <interface name="org.freedesktop.DBus.Peer">
#     <method name="Ping"/>
#     <method name="GetMachineId">
#       <arg type="s" name="machine_uuid" direction="out"/>
#     </method>
#   </interface>
#   <interface name="org.freedesktop.GeoClue2.Client">
#     <method name="Start"/>
#     <method name="Stop"/>
#     <signal name="LocationUpdated">
#       <arg type="o" name="old"/>
#       <arg type="o" name="new"/>
#     </signal>
#     <property type="o" name="Location" access="read"/>
#     <property type="u" name="DistanceThreshold" access="readwrite">
#       <annotation name="org.freedesktop.Accounts.DefaultValue" value="0"/>
#     </property>
#     <property type="u" name="TimeThreshold" access="readwrite">
#       <annotation name="org.freedesktop.Accounts.DefaultValue" value="0"/>
#     </property>
#     <property type="s" name="DesktopId" access="readwrite"/>
#     <property type="u" name="RequestedAccuracyLevel" access="readwrite"/>
#     <property type="b" name="Active" access="read"/>
#   </interface>
#   <node name="Location"/>
# </node>
#
# bus = SessionBus()
# fake_desc = bus.get('org.freedesktop.woot', '/org/freedesktop/GeoClue2/Manager').Introspect()
# print(fake_desc)
# print(bus.get('org.freedesktop.woot', '/org/freedesktop/GeoClue2/Manager').GetClient())
# # assert fake_desc == manager_desc


# bus = SystemBus()
# print(bus.get('.GeoClue2', '/org/freedesktop/GeoClue2/Client/1/Location/0').Introspect())

# sys.exit(0)


# bus = SystemBus()
bus = SessionBus()
loop = GLib.MainLoop()

manager = bus.get('org.freedesktop.GeoClue2', '/org/freedesktop/GeoClue2/Manager')
client_addr = manager.GetClient()
print('Geoclue client path %r' % (client_addr,))

client = bus.get('org.freedesktop.GeoClue2', client_addr)
# print(client.Introspect())

print('client GetAll', repr(client.GetAll('org.freedesktop.GeoClue2.Client')))
client.Set('org.freedesktop.GeoClue2.Client', 'DesktopId', Variant('s', 'w00t'))
client.Set('org.freedesktop.GeoClue2.Client', 'DistanceThreshold', Variant('u', 100000))


def location_signal(sender, object, iface, signal, params):
    # print('location_signal', repr((sender, object, iface, signal, params)))
    # print('yo', bus.get('.GeoClue2', params[1]).Introspect())
    location = bus.get('.GeoClue2', params[1]).GetAll("org.freedesktop.GeoClue2.Location")
    print('Location', repr(location))


bus.subscribe(signal='LocationUpdated', signal_fired=location_signal)

resp = client.Start()
loop.run()
