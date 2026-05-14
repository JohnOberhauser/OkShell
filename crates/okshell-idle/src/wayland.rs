use std::collections::HashMap;
use std::fmt;

use gtk::gdk;
use gtk::prelude::{Cast, NativeExt};
use relm4::gtk;

use gdk4_wayland::{WaylandDisplay, WaylandSurface};

use gdk4_wayland::prelude::WaylandSurfaceExtManual;
use wayland_client::protocol::{wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy};
use wayland_protocols::wp::idle_inhibit::zv1::client::{
    zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1, zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1,
};

pub type SurfaceId = String;

struct AppData {
    manager: Option<ZwpIdleInhibitManagerV1>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "zwp_idle_inhibit_manager_v1" {
                let v = version.min(1);
                let manager = proxy.bind::<ZwpIdleInhibitManagerV1, _, _>(name, v, qh, ());
                state.manager = Some(manager);
            }
        }
    }
}

impl Dispatch<ZwpIdleInhibitManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ZwpIdleInhibitManagerV1,
        _: <ZwpIdleInhibitManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpIdleInhibitorV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &ZwpIdleInhibitorV1,
        _: <ZwpIdleInhibitorV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

struct ActiveInhibitor {
    inhibitor: ZwpIdleInhibitorV1,
    anchor: SurfaceId,
}

impl Drop for ActiveInhibitor {
    fn drop(&mut self) {
        self.inhibitor.destroy();
    }
}

pub struct IdleManager {
    connection: Connection,
    queue: EventQueue<AppData>,
    data: AppData,

    intent: bool,
    surfaces: HashMap<SurfaceId, wl_surface::WlSurface>,
    active: Option<ActiveInhibitor>,
}

impl IdleManager {
    pub fn new() -> Result<Self, IdleError> {
        let display = gdk::Display::default()
            .ok_or(IdleError::NoDisplay)?
            .downcast::<WaylandDisplay>()
            .map_err(|_| IdleError::NotWayland)?;

        let wl_display = display.wl_display().ok_or(IdleError::NoWlDisplay)?;

        // The WlDisplay proxy GDK gave us is already on a Connection — pull it out.
        let connection = Connection::from_backend(wl_display.backend().upgrade().unwrap());

        let mut queue: EventQueue<AppData> = connection.new_event_queue();
        let qh = queue.handle();
        connection.display().get_registry(&qh, ());

        let mut data = AppData { manager: None };
        queue.roundtrip(&mut data).map_err(IdleError::Roundtrip)?;

        if data.manager.is_none() {
            return Err(IdleError::NoInhibitManager);
        }

        Ok(Self {
            connection,
            queue,
            data,
            intent: false,
            surfaces: HashMap::new(),
            active: None,
        })
    }

    pub fn set_inhibit(&mut self, on: bool) {
        if self.intent == on {
            return;
        }
        self.intent = on;
        self.reconcile();
    }

    pub fn register_surface(
        &mut self,
        id: SurfaceId,
        window: &gtk::Window,
    ) -> Result<(), IdleError> {
        let gdk_surface = window.surface().ok_or(IdleError::NoGdkSurface)?;
        let wl_surface = self.adopt_gdk_surface(&gdk_surface)?;

        self.surfaces.insert(id, wl_surface);
        self.reconcile();
        Ok(())
    }

    pub fn unregister_surface(&mut self, id: &SurfaceId) {
        if self.surfaces.remove(id).is_none() {
            return;
        }

        if matches!(&self.active, Some(a) if a.anchor == *id) {
            self.active = None;
        }

        self.reconcile();
        let _ = self.connection.flush();
    }

    fn reconcile(&mut self) {
        match (self.intent, self.active.is_some()) {
            (false, false) => {}
            (false, true) => {
                self.active = None;
            }
            (true, true) => {
                let anchor_alive = self
                    .active
                    .as_ref()
                    .map(|a| self.surfaces.contains_key(&a.anchor))
                    .unwrap_or(false);
                if !anchor_alive {
                    self.active = None;
                    self.create_inhibitor_on_any();
                }
            }
            (true, false) => {
                self.create_inhibitor_on_any();
            }
        }

        let _ = self.connection.flush();
    }

    fn create_inhibitor_on_any(&mut self) {
        let Some(manager) = &self.data.manager else {
            return;
        };
        let Some((id, surface)) = self.surfaces.iter().next() else {
            return;
        };

        let inhibitor = manager.create_inhibitor(surface, &self.queue.handle(), ());
        self.active = Some(ActiveInhibitor {
            inhibitor,
            anchor: id.clone(),
        });
    }

    fn adopt_gdk_surface(
        &self,
        gdk_surface: &gdk::Surface,
    ) -> Result<wl_surface::WlSurface, IdleError> {
        let wayland_surface = gdk_surface
            .downcast_ref::<WaylandSurface>()
            .ok_or(IdleError::NotWaylandSurface)?;

        wayland_surface.wl_surface().ok_or(IdleError::NoWlSurface)
    }
}

#[derive(Debug)]
pub enum IdleError {
    NoDisplay,
    NotWayland,
    NoWlDisplay,
    Roundtrip(wayland_client::DispatchError),
    NoInhibitManager,
    NoGdkSurface,
    NotWaylandSurface,
    NoWlSurface,
}

impl fmt::Display for IdleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDisplay => f.write_str("no default GDK display"),
            Self::NotWayland => f.write_str("GDK display is not a Wayland display"),
            Self::Roundtrip(e) => write!(f, "Wayland roundtrip failed: {e}"),
            Self::NoInhibitManager => {
                f.write_str("compositor does not advertise zwp_idle_inhibit_manager_v1")
            }
            Self::NoGdkSurface => {
                f.write_str("GTK window has no GdkSurface yet (call after realize)")
            }
            Self::NotWaylandSurface => f.write_str("GdkSurface is not a WaylandSurface"),
            Self::NoWlSurface => f.write_str("WaylandSurface has no wl_surface"),
            Self::NoWlDisplay => f.write_str("WaylandDisplay has no wl_display"),
        }
    }
}

impl std::error::Error for IdleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Roundtrip(e) => Some(e),
            _ => None,
        }
    }
}
