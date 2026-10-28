// These are native application WebViews, not general-purpose browsers.
// Suppress the browser engine's navigation/print/save/share context menu.
// In-app resource action menus remain available through their own buttons.
document.addEventListener("contextmenu",event=>event.preventDefault(),{capture:true});
