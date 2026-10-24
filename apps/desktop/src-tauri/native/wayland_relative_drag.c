// GTK3 Wayland relative-pointer-v1 bridge. All functions and events run on
// GTK's main thread, using GDK's existing display, seat and pointer.
// No independent Wayland connection, socket reader or input grab.
#include <gdk/gdk.h>
#include <gdk/gdkwayland.h>
#include <wayland-client.h>
#include <string.h>
#include "relative-pointer-unstable-v1-client-protocol.h"

typedef void (*cfvigil_motion_fn)(double dx, double dy);

static struct {
    struct wl_display *display;            // owned by GDK
    struct wl_pointer *pointer;            // owned by GDK; never destroy
    struct wl_registry *registry;          // owned by this bridge
    struct zwp_relative_pointer_manager_v1 *manager;
    struct zwp_relative_pointer_v1 *relative;
    uint32_t global_id;
    cfvigil_motion_fn on_motion;
    int active;
} drag;

static void relative_motion(void *unused, struct zwp_relative_pointer_v1 *relative,
                            uint32_t hi, uint32_t lo, wl_fixed_t dx, wl_fixed_t dy,
                            wl_fixed_t dx_unaccel, wl_fixed_t dy_unaccel) {
    (void)unused;
    (void)relative;
    (void)hi;
    (void)lo;
    (void)dx_unaccel;
    (void)dy_unaccel;
    if (drag.active && drag.on_motion) {
        drag.on_motion(wl_fixed_to_double(dx), wl_fixed_to_double(dy));
    }
}

static const struct zwp_relative_pointer_v1_listener relative_listener = {
    relative_motion
};

static void registry_global(void *unused, struct wl_registry *registry,
                            uint32_t id, const char *interface, uint32_t version) {
    (void)unused;
    (void)version;
    if (strcmp(interface, zwp_relative_pointer_manager_v1_interface.name) != 0
        || drag.manager || !drag.pointer) {
        return;
    }

    drag.manager = wl_registry_bind(registry, id,
                                    &zwp_relative_pointer_manager_v1_interface, 1);
    if (!drag.manager) return;
    drag.global_id = id;
    drag.relative = zwp_relative_pointer_manager_v1_get_relative_pointer(
        drag.manager, drag.pointer);
    if (!drag.relative ||
        zwp_relative_pointer_v1_add_listener(drag.relative, &relative_listener, NULL) != 0) {
        drag.active = 0;
        if (drag.relative) zwp_relative_pointer_v1_destroy(drag.relative);
        drag.relative = NULL;
        zwp_relative_pointer_manager_v1_destroy(drag.manager);
        drag.manager = NULL;
    }
}

static void registry_remove(void *unused, struct wl_registry *registry, uint32_t id) {
    (void)unused;
    (void)registry;
    if (id != drag.global_id) return;

    drag.active = 0;
    if (drag.relative) zwp_relative_pointer_v1_destroy(drag.relative);
    drag.relative = NULL;
    if (drag.manager) zwp_relative_pointer_manager_v1_destroy(drag.manager);
    drag.manager = NULL;
    drag.global_id = 0;
}

static const struct wl_registry_listener registry_listener = {
    registry_global, registry_remove
};

// Returns 1 if protocol discovery was successfully queued. The actual global
// binding happens via the GDK-owned Wayland event loop, asynchronously.
int cfvigil_relative_init(cfvigil_motion_fn on_motion) {
    if (drag.registry || !on_motion) return 0;
    GdkDisplay *display = gdk_display_get_default();
    if (!display || !GDK_IS_WAYLAND_DISPLAY(display)) return 0;
    GdkSeat *seat = gdk_display_get_default_seat(display);
    if (!seat) return 0;
    GdkDevice *device = gdk_seat_get_pointer(seat);
    if (!device || !GDK_IS_WAYLAND_DEVICE(device)) return 0;

    struct wl_pointer *pointer = gdk_wayland_device_get_wl_pointer(device);
    struct wl_display *wayland = gdk_wayland_display_get_wl_display(display);
    if (!pointer || !wayland) return 0;
    drag.pointer = pointer;
    drag.display = wayland;
    drag.on_motion = on_motion;
    drag.registry = wl_display_get_registry(wayland);
    if (!drag.registry ||
        wl_registry_add_listener(drag.registry, &registry_listener, NULL) != 0) {
        if (drag.registry) wl_registry_destroy(drag.registry);
        drag.registry = NULL;
        drag.pointer = NULL;
        drag.display = NULL;
        drag.on_motion = NULL;
        return 0;
    }
    // The GTK/GDK event loop owns dispatch. Do not roundtrip or read GDK's fd.
    return wl_display_flush(wayland) >= 0;
}

// The normal cursor stays visible. Motion arrives only with pointer focus,
// which Wayland's implicit left-button grab retains during a drag.
int cfvigil_relative_begin(void) {
    if (!drag.relative || !drag.manager || drag.active) return 0;
    drag.active = 1;
    return 1;
}

void cfvigil_relative_end(void) {
    drag.active = 0;
}
