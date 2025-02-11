mod imp {
    use std::cell::RefCell;
    use std::{cell::Cell, rc::Rc};

    use adw::{prelude::*, subclass::prelude::*};
    use gtk4::{
        gdk, gio, glib, glib::clone, subclass::prelude::*, Box, CompositeTemplate, CssProvider,
        FileChooserNative, Grid, Inhibit, Overlay, PackType, Picture, ScrolledWindow, StyleContext,
        ToggleButton,
    };
    use gtk4::{GestureDrag, PropagationPhase, Revealer, Separator};

    use crate::{
        app::RnoteApp, config, ui::canvas::Canvas, ui::develactions::DevelActions, ui::dialogs,
        ui::mainheader::MainHeader, ui::penssidebar::PensSideBar,
        ui::selectionmodifier::SelectionModifier, ui::settingspanel::SettingsPanel,
        ui::workspacebrowser::WorkspaceBrowser,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/appwindow.ui")]
    pub struct RnoteAppWindow {
        pub settings: gio::Settings,
        pub filechoosernative: Rc<RefCell<Option<FileChooserNative>>>,
        #[template_child]
        pub main_grid: TemplateChild<Grid>,
        #[template_child]
        pub devel_actions_revealer: TemplateChild<Revealer>,
        #[template_child]
        pub devel_actions: TemplateChild<DevelActions>,
        #[template_child]
        pub canvas_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub canvas: TemplateChild<Canvas>,
        #[template_child]
        pub canvas_overlay: TemplateChild<Overlay>,
        #[template_child]
        pub canvas_resize_preview: TemplateChild<Picture>,
        #[template_child]
        pub settings_panel: TemplateChild<SettingsPanel>,
        #[template_child]
        pub selection_modifier: TemplateChild<SelectionModifier>,
        #[template_child]
        pub sidebar_grid: TemplateChild<Grid>,
        #[template_child]
        pub sidebar_sep: TemplateChild<Separator>,
        #[template_child]
        pub flap: TemplateChild<adw::Flap>,
        #[template_child]
        pub flap_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub flap_header: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub flap_resizer: TemplateChild<gtk4::Box>,
        #[template_child]
        pub flap_resizer_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub workspacebrowser: TemplateChild<WorkspaceBrowser>,
        #[template_child]
        pub flapreveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub flap_menus_box: TemplateChild<Box>,
        #[template_child]
        pub mainheader: TemplateChild<MainHeader>,
        #[template_child]
        pub penssidebar: TemplateChild<PensSideBar>,
    }

    impl Default for RnoteAppWindow {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(config::APP_ID),
                filechoosernative: Rc::new(RefCell::new(None)),
                main_grid: TemplateChild::<Grid>::default(),
                devel_actions_revealer: TemplateChild::<Revealer>::default(),
                devel_actions: TemplateChild::<DevelActions>::default(),
                canvas_scroller: TemplateChild::<ScrolledWindow>::default(),
                canvas: TemplateChild::<Canvas>::default(),
                canvas_overlay: TemplateChild::<Overlay>::default(),
                canvas_resize_preview: TemplateChild::<Picture>::default(),
                settings_panel: TemplateChild::<SettingsPanel>::default(),
                selection_modifier: TemplateChild::<SelectionModifier>::default(),
                sidebar_grid: TemplateChild::<Grid>::default(),
                sidebar_sep: TemplateChild::<Separator>::default(),
                flap: TemplateChild::<adw::Flap>::default(),
                flap_box: TemplateChild::<gtk4::Box>::default(),
                flap_header: TemplateChild::<adw::HeaderBar>::default(),
                flap_resizer: TemplateChild::<gtk4::Box>::default(),
                flap_resizer_box: TemplateChild::<gtk4::Box>::default(),
                workspacebrowser: TemplateChild::<WorkspaceBrowser>::default(),
                flapreveal_toggle: TemplateChild::<ToggleButton>::default(),
                flap_menus_box: TemplateChild::<Box>::default(),
                mainheader: TemplateChild::<MainHeader>::default(),
                penssidebar: TemplateChild::<PensSideBar>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteAppWindow {
        const NAME: &'static str = "RnoteAppWindow";
        type Type = super::RnoteAppWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnoteAppWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let flap = self.flap.get();
            let flap_box = self.flap_box.get();
            let flap_resizer = self.flap_resizer.get();
            let flap_resizer_box = self.flap_resizer_box.get();
            let workspace_headerbar = self.flap_header.get();
            let flapreveal_toggle = self.flapreveal_toggle.get();

            let _windowsettings = obj.settings();
            //windowsettings.set_gtk_application_prefer_dark_theme(true);

            flap.set_locked(true);
            flap.set_fold_policy(adw::FlapFoldPolicy::Auto);

            let css = CssProvider::new();
            css.load_from_resource((String::from(config::APP_IDPATH) + "ui/custom.css").as_str());

            let display = gdk::Display::default().unwrap();
            StyleContext::add_provider_for_display(
                &display,
                &css,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            let expanded_revealed = Rc::new(Cell::new(flap.reveals_flap()));

            self.flapreveal_toggle
                .bind_property("active", &flap, "reveal-flap")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();

            self.flapreveal_toggle.connect_toggled(
                clone!(@weak flap, @strong expanded_revealed => move |flapreveal_toggle| {
                    flap.set_reveal_flap(flapreveal_toggle.is_active());
                    if !flap.is_folded() {
                        expanded_revealed.set(flapreveal_toggle.is_active());
                    }
                }),
            );

            self.flap
                .connect_folded_notify(clone!(@strong expanded_revealed, @weak flapreveal_toggle, @weak workspace_headerbar => move |flap| {
                    if flap.is_folded() {
                        flapreveal_toggle.set_active(false);
                    } else {
                        if flap.flap_position() == PackType::End {
                            workspace_headerbar.set_show_end_title_buttons(flap.reveals_flap());
                        }
                        if expanded_revealed.get() || flap.reveals_flap() {
                            expanded_revealed.set(true);
                            flapreveal_toggle.set_active(true);
                        }
                    }
                }));

            self.flap
                .connect_reveal_flap_notify(clone!(@weak workspace_headerbar => move |flap| {
                    if !flap.is_folded() && flap.flap_position() == PackType::End {
                        workspace_headerbar.set_show_end_title_buttons(flap.reveals_flap());
                    } else {
                        workspace_headerbar.set_show_end_title_buttons(false);
                    }
                }));

            self.flap.connect_flap_position_notify(
                clone!(@weak workspace_headerbar, @strong expanded_revealed => move |flap| {
                    if !flap.is_folded() && flap.flap_position() == PackType::End {
                        workspace_headerbar.set_show_end_title_buttons(expanded_revealed.get());
                    } else {
                        workspace_headerbar.set_show_end_title_buttons(false);
                    }
                }),
            );

            // Resizing the flap contents
            let resizer_drag_gesture = GestureDrag::builder()
                .name("resizer_drag_gesture")
                .propagation_phase(PropagationPhase::Capture)
                .build();
            self.flap_resizer.add_controller(&resizer_drag_gesture);

            // Dirty hack to stop resizing when it is switching from non-folded to folded or vice versa
            let prev_folded = Rc::new(Cell::new(flap.is_folded()));

            resizer_drag_gesture.connect_drag_begin(clone!(@strong prev_folded, @weak flap, @weak flap_box => move |_resizer_drag_gesture, _x , _y| {
                    prev_folded.set(flap.is_folded());
            }));

            resizer_drag_gesture.connect_drag_update(clone!(@weak obj, @strong prev_folded, @weak flap, @weak flap_box, @weak flapreveal_toggle => move |_resizer_drag_gesture, x , _y| {
                if flap.is_folded() == prev_folded.get() {
                    // Set BEFORE new width request
                    prev_folded.set(flap.is_folded());

                    let new_width = if flap.flap_position() == PackType::Start {
                        flap_box.width() + x.ceil() as i32
                    } else {
                        flap_box.width() - x.floor() as i32
                    };
                    if new_width > 0 && new_width < obj.mainheader().width() - 64 {
                        flap_box.set_width_request(new_width);
                    }
                } else {
                    if flap.is_folded() {
                        flapreveal_toggle.set_active(true);
                    }
                }
            }));

            self.flap_resizer.set_cursor(
                gdk::Cursor::from_name(
                    "col-resize",
                    gdk::Cursor::from_name("default", None).as_ref(),
                )
                .as_ref(),
            );

            self.flap.get().connect_flap_position_notify(
                clone!(@weak flap_resizer_box, @weak flap_resizer, @weak flap_box => move |flap| {
                    if flap.flap_position() == PackType::Start {
                            flap_resizer_box.remove::<gtk4::Box>(&flap_box);
                            flap_resizer_box.remove::<gtk4::Box>(&flap_resizer);
                            flap_resizer_box.prepend::<gtk4::Box>(&flap_box);
                            flap_resizer_box.append::<gtk4::Box>(&flap_resizer);
                    } else {
                            flap_resizer_box.remove::<gtk4::Box>(&flap_resizer);
                            flap_resizer_box.remove::<gtk4::Box>(&flap_box);
                            flap_resizer_box.prepend::<gtk4::Box>(&flap_resizer);
                            flap_resizer_box.append::<gtk4::Box>(&flap_box);
                    }
                }),
            );

            // Load latest window state
            obj.load_window_size();
        }
    }

    impl WidgetImpl for RnoteAppWindow {}

    impl WindowImpl for RnoteAppWindow {
        // Save window state right before the window will be closed
        fn close_request(&self, obj: &Self::Type) -> Inhibit {
            if let Err(err) = obj.save_window_size() {
                log::error!("Failed to save window state, {}", &err);
            }

            // Save current sheet
            if obj
                .application()
                .unwrap()
                .downcast::<RnoteApp>()
                .unwrap()
                .unsaved_changes()
            {
                dialogs::dialog_quit_save(obj);
            } else {
                obj.destroy();
            }
            // Inhibit (Overwrite) the default handler. This handler is then responsible for destoying the window.
            Inhibit(true)
        }
    }

    impl ApplicationWindowImpl for RnoteAppWindow {}
    impl AdwWindowImpl for RnoteAppWindow {}
    impl AdwApplicationWindowImpl for RnoteAppWindow {}
}

use std::{
    boxed,
    cell::{Cell, RefCell},
    error::Error,
    path::PathBuf,
    rc::Rc,
};

use adw::prelude::*;
use gtk4::{
    gdk, gio, glib, glib::clone, graphene, subclass::prelude::*, Application, Box,
    EventControllerScroll, EventControllerScrollFlags, FileChooserNative, GestureDrag, GestureZoom,
    Grid, Inhibit, Overlay, Picture, PropagationPhase, Revealer, ScrolledWindow, Separator,
    Snapshot, ToggleButton,
};

use crate::{
    app::RnoteApp,
    strokes::{bitmapimage::BitmapImage, vectorimage::VectorImage, StrokeStyle},
    ui::canvas::Canvas,
    ui::develactions::DevelActions,
    ui::penssidebar::PensSideBar,
    ui::settingspanel::SettingsPanel,
    ui::{actions, selectionmodifier::SelectionModifier, workspacebrowser::WorkspaceBrowser},
    ui::{dialogs, mainheader::MainHeader},
    utils,
};

glib::wrapper! {
    pub struct RnoteAppWindow(ObjectSubclass<imp::RnoteAppWindow>)
        @extends gtk4::Widget, gtk4::Window, adw::Window, gtk4::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl RnoteAppWindow {
    pub const CANVAS_ZOOMGESTURE_THRESHOLD: f64 = 0.005; // Sets the delta threshold (eg. 0.01 = 1% ) when to update the canvas when doing a zoom gesture
    pub const CANVAS_ZOOM_SCROLL_STEP: f64 = 0.1; // Sets the canvas zoom scroll step in
    pub const CANVAS_ZOOMGESTURE_ZOOM_SPEED: f64 = 0.8; // Sets the canvas zoom speed, 1.0 for one-to-one scale_delta to zoom ratio

    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create `RnoteAppWindow`.")
    }

    pub fn app_settings(&self) -> &gio::Settings {
        &imp::RnoteAppWindow::from_instance(self).settings
    }

    pub fn filechoosernative(&self) -> Rc<RefCell<Option<FileChooserNative>>> {
        imp::RnoteAppWindow::from_instance(self)
            .filechoosernative
            .clone()
    }

    pub fn main_grid(&self) -> Grid {
        imp::RnoteAppWindow::from_instance(self).main_grid.get()
    }

    pub fn devel_actions_revealer(&self) -> Revealer {
        imp::RnoteAppWindow::from_instance(self)
            .devel_actions_revealer
            .get()
    }

    pub fn devel_actions(&self) -> DevelActions {
        imp::RnoteAppWindow::from_instance(self).devel_actions.get()
    }

    pub fn canvas_scroller(&self) -> ScrolledWindow {
        imp::RnoteAppWindow::from_instance(self)
            .canvas_scroller
            .get()
    }

    pub fn canvas_overlay(&self) -> Overlay {
        imp::RnoteAppWindow::from_instance(self)
            .canvas_overlay
            .get()
    }

    pub fn canvas(&self) -> Canvas {
        imp::RnoteAppWindow::from_instance(self).canvas.get()
    }

    pub fn canvas_resize_preview(&self) -> Picture {
        imp::RnoteAppWindow::from_instance(self)
            .canvas_resize_preview
            .get()
    }

    pub fn settings_panel(&self) -> SettingsPanel {
        imp::RnoteAppWindow::from_instance(self)
            .settings_panel
            .get()
    }

    pub fn selection_modifier(&self) -> SelectionModifier {
        imp::RnoteAppWindow::from_instance(self)
            .selection_modifier
            .get()
    }

    pub fn sidebar_grid(&self) -> Grid {
        imp::RnoteAppWindow::from_instance(self).sidebar_grid.get()
    }

    pub fn sidebar_sep(&self) -> Separator {
        imp::RnoteAppWindow::from_instance(self).sidebar_sep.get()
    }

    pub fn flap_header(&self) -> adw::HeaderBar {
        imp::RnoteAppWindow::from_instance(self).flap_header.get()
    }

    pub fn workspacebrowser(&self) -> WorkspaceBrowser {
        imp::RnoteAppWindow::from_instance(self)
            .workspacebrowser
            .get()
    }

    pub fn flap(&self) -> adw::Flap {
        imp::RnoteAppWindow::from_instance(self).flap.get()
    }

    pub fn flapreveal_toggle(&self) -> ToggleButton {
        imp::RnoteAppWindow::from_instance(self)
            .flapreveal_toggle
            .get()
    }

    pub fn flap_menus_box(&self) -> Box {
        imp::RnoteAppWindow::from_instance(self)
            .flap_menus_box
            .get()
    }

    pub fn mainheader(&self) -> MainHeader {
        imp::RnoteAppWindow::from_instance(self).mainheader.get()
    }

    pub fn penssidebar(&self) -> PensSideBar {
        imp::RnoteAppWindow::from_instance(self).penssidebar.get()
    }

    pub fn set_color_scheme(&self, color_scheme: adw::ColorScheme) {
        self.application()
            .unwrap()
            .downcast::<RnoteApp>()
            .unwrap()
            .style_manager()
            .set_color_scheme(color_scheme);

        match color_scheme {
            adw::ColorScheme::Default => {
                self.app_settings()
                    .set_string("color-scheme", "default")
                    .unwrap();
                self.mainheader()
                    .appmenu()
                    .default_theme_toggle()
                    .set_active(true);
            }
            adw::ColorScheme::ForceLight => {
                self.app_settings()
                    .set_string("color-scheme", "force-light")
                    .unwrap();
                self.mainheader()
                    .appmenu()
                    .light_theme_toggle()
                    .set_active(true);
            }
            adw::ColorScheme::ForceDark => {
                self.app_settings()
                    .set_string("color-scheme", "force-dark")
                    .unwrap();
                self.mainheader()
                    .appmenu()
                    .dark_theme_toggle()
                    .set_active(true);
            }
            _ => {
                log::error!("unsupported color_scheme in set_color_scheme()");
            }
        }
    }

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = &imp::RnoteAppWindow::from_instance(self).settings;

        let mut width = self.width();
        let mut height = self.height();

        // Window would grow without subtracting this size. Why? I dont know
        width -= 122;
        height -= 122;

        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;
        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let width = self.app_settings().int("window-width");
        let height = self.app_settings().int("window-height");
        let is_maximized = self.app_settings().boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    /// The point parameter has the coordinate space of the sheet, NOT of the scrolled window!
    pub fn canvas_scroller_center_around_point_on_sheet(&self, point: (f64, f64)) {
        let scroller_dimensions = (
            f64::from(self.canvas_scroller().width()),
            f64::from(self.canvas_scroller().height()),
        );
        let canvas_dimensions = (
            f64::from(self.canvas().width()),
            f64::from(self.canvas().height()),
        );

        if canvas_dimensions.0 > scroller_dimensions.0 {
            self.canvas_scroller()
                .hadjustment()
                .unwrap()
                .set_value((point.0 * self.canvas().scalefactor()) - scroller_dimensions.0 * 0.5);
        }
        if canvas_dimensions.1 > scroller_dimensions.1 {
            self.canvas_scroller()
                .vadjustment()
                .unwrap()
                .set_value((point.1 * self.canvas().scalefactor()) - scroller_dimensions.1 * 0.5);
        }
    }

    pub fn canvas_scroller_viewport(&self) -> Option<p2d::bounding_volume::AABB> {
        let pos = if let (Some(hadjustment), Some(vadjustment)) = (
            self.canvas_scroller().hadjustment(),
            self.canvas_scroller().vadjustment(),
        ) {
            na::vector![hadjustment.value(), vadjustment.value()]
        } else {
            return None;
        };
        let width = f64::from(self.canvas_scroller().width());
        let height = f64::from(self.canvas_scroller().height());
        Some(p2d::bounding_volume::AABB::new(
            na::Point2::<f64>::from(pos),
            na::point![pos[0] + width, pos[1] + height],
        ))
    }

    // Must be called after application is associated with it else it fails
    pub fn init(&self) {
        let priv_ = imp::RnoteAppWindow::from_instance(self);

        priv_.workspacebrowser.get().init(self);
        priv_.canvas.get().init(self);
        priv_.mainheader.get().init(self);
        priv_.mainheader.get().canvasmenu().init(self);
        priv_.mainheader.get().appmenu().init(self);
        priv_.penssidebar.get().init(self);
        priv_.canvas.get().sheet().selection().init(self);
        priv_.settings_panel.get().init(self);
        priv_.selection_modifier.get().init(self);
        priv_.devel_actions.get().init(self);

        // Loading in input file
        if let Some(input_file) = self
            .application()
            .unwrap()
            .downcast::<RnoteApp>()
            .unwrap()
            .input_file()
            .borrow()
            .to_owned()
        {
            if self
                .application()
                .unwrap()
                .downcast::<RnoteApp>()
                .unwrap()
                .unsaved_changes()
            {
                dialogs::dialog_open_overwrite(self);
            } else if let Err(e) = self.load_in_file(&input_file) {
                log::error!("failed to load in input file, {}", e);
            }
        }

        self.flap_header().connect_show_end_title_buttons_notify(
            clone!(@weak self as appwindow => move |_files_headerbar| {
                if appwindow.flap_header().shows_end_title_buttons() {
                    //appwindow.mainheader().menus_box().remove(&appwindow.mainheader().canvasmenu());
                    appwindow.mainheader().menus_box().remove(&appwindow.mainheader().appmenu());
                    //appwindow.flap_menus_box().append(&appwindow.mainheader().canvasmenu());
                    appwindow.flap_menus_box().append(&appwindow.mainheader().appmenu());
                } else {
                    //appwindow.flap_menus_box().remove(&appwindow.mainheader().canvasmenu());
                    appwindow.flap_menus_box().remove(&appwindow.mainheader().appmenu());
                    //appwindow.mainheader().menus_box().append(&appwindow.mainheader().canvasmenu());
                    appwindow.mainheader().menus_box().append(&appwindow.mainheader().appmenu());
                }
            }),
        );

        // zoom scrolling with <ctrl> + scroll
        let canvas_zoom_scroll_controller = EventControllerScroll::builder()
            .name("canvas_zoom_scroll_controller")
            .propagation_phase(PropagationPhase::Capture)
            .flags(EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::DISCRETE)
            .build();
        canvas_zoom_scroll_controller.connect_scroll(clone!(@weak self as appwindow => @default-return Inhibit(false), move |zoom_scroll_controller, _dx, dy| {
            if zoom_scroll_controller.current_event_state() == gdk::ModifierType::CONTROL_MASK {
                let delta = dy * (Self::CANVAS_ZOOM_SCROLL_STEP * appwindow.canvas().scalefactor());
                let new_scalefactor = appwindow.canvas().scalefactor() - delta;

                // the sheet position BEFORE scaling
                let sheet_center_pos = (
                    ((appwindow.canvas_scroller().hadjustment().unwrap().value()
                    + f64::from(appwindow.canvas_scroller().width()) * 0.5) / appwindow.canvas().scalefactor()) + f64::from(appwindow.canvas().sheet().x()),
                    ((appwindow.canvas_scroller().vadjustment().unwrap().value()
                    + f64::from(appwindow.canvas_scroller().height()) * 0.5) / appwindow.canvas().scalefactor()) + f64::from(appwindow.canvas().sheet().y())
                );

                appwindow.canvas().set_scalefactor(new_scalefactor);

                // Reposition scroller center to the stored sheet position
                appwindow.canvas_scroller_center_around_point_on_sheet(sheet_center_pos);
                // Stop event propagation
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        }));
        self.canvas_scroller()
            .add_controller(&canvas_zoom_scroll_controller);

        // Move Canvas with middle mouse button
        let canvas_move_drag_gesture = GestureDrag::builder()
            .name("canvas_move_drag_gesture")
            .button(gdk::BUTTON_MIDDLE)
            .propagation_phase(PropagationPhase::Capture)
            .build();

        let move_start_x = Rc::new(Cell::new(0.0));
        let move_start_y = Rc::new(Cell::new(0.0));

        canvas_move_drag_gesture.connect_drag_begin(clone!(@strong move_start_x, @strong move_start_y, @weak self as appwindow => move |_canvas_move_motion_controller, _x, _y| {
            move_start_x.set(appwindow.canvas_scroller().hadjustment().unwrap().value());
            move_start_y.set(appwindow.canvas_scroller().vadjustment().unwrap().value());
        }));
        canvas_move_drag_gesture.connect_drag_update(clone!(@strong move_start_x, @strong move_start_y, @weak self as appwindow => move |_canvas_move_motion_controller, x, y| {
            appwindow.canvas_scroller().hadjustment().unwrap().set_value(move_start_x.get() - x);
            appwindow.canvas_scroller().vadjustment().unwrap().set_value(move_start_y.get() - y);
        }));
        self.canvas_scroller()
            .add_controller(&canvas_move_drag_gesture);

        // Canvas gesture zooming with preview and dragging
        let canvas_zoom_gesture = GestureZoom::builder()
            .name("canvas_zoom_gesture")
            .propagation_phase(PropagationPhase::Capture)
            .build();
        self.canvas_scroller().add_controller(&canvas_zoom_gesture);

        let scale_begin = Rc::new(Cell::new(1_f64));
        let scale_doubledelta = Rc::new(Cell::new(1_f64));
        let canvas_preview_paintable = Rc::new(RefCell::new(gdk::Paintable::new_empty(0, 0)));
        let zoomgesture_canvasscroller_start_pos = Rc::new(Cell::new((0.0, 0.0)));
        let zoomgesture_bbcenter_start: Rc<Cell<Option<(f64, f64)>>> = Rc::new(Cell::new(None));

        canvas_zoom_gesture.connect_begin(
            clone!(
                @strong canvas_preview_paintable,
                @strong scale_begin,
                @strong scale_doubledelta,
                @strong zoomgesture_canvasscroller_start_pos,
                @strong zoomgesture_bbcenter_start,
                @weak self as appwindow => move |canvas_zoom_gesture, _eventsequence| {
                scale_begin.set(appwindow.canvas().scalefactor());
                scale_doubledelta.set(1_f64);

                let width = f64::from(appwindow.canvas().sheet().width()) * scale_begin.get();
                let height = f64::from(appwindow.canvas().sheet().height()) * scale_begin.get();
                let preview_size = graphene::Size::new(width as f32, height as f32);

                zoomgesture_canvasscroller_start_pos.set(
                    (
                        appwindow.canvas_scroller().hadjustment().unwrap().value(),
                        appwindow.canvas_scroller().vadjustment().unwrap().value()
                    )
                );
                if let Some(bbcenter) = canvas_zoom_gesture.bounding_box_center() {
                    zoomgesture_bbcenter_start.set(Some(
                        bbcenter
                    ));
                }

                *canvas_preview_paintable.borrow_mut() = appwindow.canvas().preview().current_image();

                if let Some(paintable) = canvas_preview_paintable.borrow().as_ref() {
                    let snapshot = Snapshot::new();
                    paintable.snapshot(snapshot.dynamic_cast_ref::<gdk::Snapshot>().unwrap(), width, height);
                    appwindow.canvas_resize_preview().set_paintable(snapshot.to_paintable(Some(&preview_size)).as_ref());
                }

                appwindow.canvas().set_visible(false);
                appwindow.canvas().sheet().selection().set_shown(false);
                appwindow.canvas_resize_preview().set_visible(true);
            }),
        );

        canvas_zoom_gesture.connect_scale_changed(
            clone!(@strong canvas_preview_paintable, @strong scale_begin, @strong scale_doubledelta, @strong zoomgesture_canvasscroller_start_pos, @strong zoomgesture_bbcenter_start, @weak self as appwindow => move |canvas_zoom_gesture, scale_delta| {
                let scale_delta = scale_delta * Self::CANVAS_ZOOMGESTURE_ZOOM_SPEED;
                let new_scalefactor = scale_begin.get() * scale_delta;

                if scale_delta < scale_doubledelta.get() - Self::CANVAS_ZOOMGESTURE_THRESHOLD || scale_delta > scale_doubledelta.get() + Self::CANVAS_ZOOMGESTURE_THRESHOLD {
                    scale_doubledelta.set(scale_delta);

                    let width = f64::from(appwindow.canvas().sheet().width()) * new_scalefactor;
                    let height = f64::from(appwindow.canvas().sheet().height()) * new_scalefactor;
                    let preview_size = graphene::Size::new(width as f32, height as f32);

                    if let Some(paintable) = canvas_preview_paintable.borrow().as_ref() {
                        let snapshot = Snapshot::new();
                        paintable.snapshot(snapshot.dynamic_cast_ref::<gdk::Snapshot>().unwrap(), width, height);
                        //snapshot.scale(scalefactor as f32, scalefactor as f32);
                        appwindow.canvas_resize_preview().set_paintable(snapshot.to_paintable(Some(&preview_size)).as_ref());
                    }
                }

                if let Some(bbcenter) = canvas_zoom_gesture.bounding_box_center() {
                    if let Some(bbcenter_start) = zoomgesture_bbcenter_start.get() {
                        let bbcenter_delta = (
                            bbcenter.0 - bbcenter_start.0 * scale_delta,
                            bbcenter.1 - bbcenter_start.1 * scale_delta
                        );

                        appwindow.canvas_scroller().hadjustment().unwrap().set_value(
                            zoomgesture_canvasscroller_start_pos.get().0 * scale_delta - bbcenter_delta.0
                        );
                        appwindow.canvas_scroller().vadjustment().unwrap().set_value(
                            zoomgesture_canvasscroller_start_pos.get().1 * scale_delta - bbcenter_delta.1
                        );
                    } else {
                        // Setting the start position if connect_scale_start didn't set it
                        zoomgesture_bbcenter_start.set(Some(
                            bbcenter
                        ));
                        log::debug!("### BEGIN DRAG ###");
                    }
                }
            }),
        );

        canvas_zoom_gesture.connect_cancel(
            clone!(@strong scale_begin, @strong zoomgesture_bbcenter_start, @weak self as appwindow => move |_gesture_zoom, _eventsequence| {
                zoomgesture_bbcenter_start.set(None);
                appwindow.canvas_resize_preview().set_visible(false);
                appwindow.canvas().set_visible(true);
                appwindow.canvas().sheet().selection().set_shown(!appwindow.canvas().sheet().selection().strokes().borrow().is_empty());

                appwindow.canvas().set_sensitive(false);
                appwindow.canvas().set_sensitive(true);
            }),
        );

        canvas_zoom_gesture.connect_end(
            clone!(@strong scale_begin, @strong scale_doubledelta, @strong zoomgesture_bbcenter_start, @weak self as appwindow => move |_gesture_zoom, _eventsequence| {
                zoomgesture_bbcenter_start.set(None);
                let scalefactor_new = scale_begin.get() * scale_doubledelta.get();
                appwindow.canvas().set_scalefactor(scalefactor_new);

                appwindow.canvas_resize_preview().set_visible(false);
                appwindow.canvas().set_visible(true);
                appwindow.canvas().sheet().selection().set_shown(!appwindow.canvas().sheet().selection().strokes().borrow().is_empty());

                appwindow.canvas().set_sensitive(false);
                appwindow.canvas().set_sensitive(true);
            }),
        );

        // This dictates the overlay children position and size
        self.canvas_overlay().connect_get_child_position(
            clone!(@weak self as appwindow => @default-return None, move |_canvas_overlay, widget| {
                 match widget.widget_name().as_str() {
                     "selection_modifier" => {
                        let selectionmodifier = widget.clone().downcast::<SelectionModifier>().unwrap();
                        let scalefactor = selectionmodifier.property("scalefactor").unwrap().get::<f64>().unwrap();

                         //Some(gdk::Rectangle {x: bounds.x().round() as i32, y: bounds.y().round() as i32, width: bounds.width().round() as i32, height: bounds.height().round() as i32})
                        if let Some(bounds) = &*appwindow.canvas().sheet().selection().bounds().borrow() {
                            let translate_node_size = ((bounds.maxs[0] - bounds.mins[0]).min( bounds.maxs[1] - bounds.mins[1] ) * scalefactor).round() as i32 - 2 * SelectionModifier::TRANSLATE_NODE_MARGIN;

                            appwindow.selection_modifier().translate_node().image().set_pixel_size(
                                translate_node_size.clamp(SelectionModifier::TRANSLATE_NODE_SIZE_MIN,
                                    SelectionModifier::TRANSLATE_NODE_SIZE_MAX
                            ));

                            Some(gdk::Rectangle {
                                x: (bounds.mins[0] * scalefactor).round() as i32 - SelectionModifier::RESIZE_NODE_SIZE,
                                y:  (bounds.mins[1] * scalefactor).round() as i32 - SelectionModifier::RESIZE_NODE_SIZE,
                                width: ((bounds.maxs[0] -  bounds.mins[0]) * scalefactor).round() as i32 + 2 * SelectionModifier::RESIZE_NODE_SIZE,
                                height: ((bounds.maxs[1] - bounds.mins[1]) * scalefactor).round() as i32 + 2 * SelectionModifier::RESIZE_NODE_SIZE,
                            })
                        } else { None }
                    },
                    _ => { None }
                }
            }),
        );

        // actions and settings AFTER widget callback declarations
        actions::setup_actions(self);
        actions::setup_accels(self);
        self.setup_settings();
    }

    // ### Settings are setup only at startup. Setting changes through gsettings / dconf might not be applied until app restarts
    fn setup_settings(&self) {
        let _priv_ = imp::RnoteAppWindow::from_instance(self);

        // overwriting theme so users can choose dark / light in appmenu
        //self.settings().set_gtk_theme_name(Some("Adwaita"));

        // Workspace directory
        self.workspacebrowser().set_primary_path(&PathBuf::from(
            self.app_settings().string("workspace-dir").as_str(),
        ));

        // color schemes
        match self.app_settings().string("color-scheme").as_str() {
            "default" => self.set_color_scheme(adw::ColorScheme::Default),
            "force-light" => self.set_color_scheme(adw::ColorScheme::ForceLight),
            "force-dark" => self.set_color_scheme(adw::ColorScheme::ForceDark),
            _ => {
                log::error!("failed to load setting color-scheme, unsupported string as key")
            }
        }

        // Ui for right / left handed writers
        self.application().unwrap().change_action_state(
            "righthanded",
            &self.app_settings().boolean("righthanded").to_variant(),
        );
        self.application()
            .unwrap()
            .activate_action("righthanded", None);
        self.application()
            .unwrap()
            .activate_action("righthanded", None);

        // Touch drawing
        self.app_settings()
            .bind("touch-drawing", &self.canvas(), "touch-drawing")
            .flags(gio::SettingsBindFlags::DEFAULT)
            .build();

        // Format borders
        self.canvas()
            .sheet()
            .set_format_borders(self.app_settings().boolean("format-borders"));

        // Autoexpand height
        let autoexpand_height = self.app_settings().boolean("autoexpand-height");
        self.canvas()
            .sheet()
            .set_autoexpand_height(autoexpand_height);
        self.mainheader()
            .pageedit_revealer()
            .set_reveal_child(!autoexpand_height);

        // Visual Debugging
        self.app_settings()
            .bind("visual-debug", &self.canvas(), "visual-debug")
            .flags(gio::SettingsBindFlags::DEFAULT)
            .build();

        // Developer mode
        self.app_settings()
            .bind(
                "devel",
                &self
                    .penssidebar()
                    .brush_page()
                    .templatechooser()
                    .predefined_template_experimental_listboxrow(),
                "visible",
            )
            .flags(gio::SettingsBindFlags::DEFAULT)
            .build();

        let action_devel_settings = self
            .application()
            .unwrap()
            .downcast::<RnoteApp>()
            .unwrap()
            .lookup_action("devel-settings")
            .unwrap();
        action_devel_settings
            .downcast::<gio::SimpleAction>()
            .unwrap()
            .set_enabled(self.app_settings().boolean("devel"));

        self.devel_actions_revealer()
            .set_reveal_child(self.app_settings().boolean("devel"));
    }

    pub fn load_in_file(&self, file: &gio::File) -> Result<(), boxed::Box<dyn Error>> {
        match utils::FileType::lookup_file_type(file) {
            utils::FileType::Rnote => {
                self.canvas().sheet().open_sheet(file)?;

                // Loading the sheet properties into the format settings panel
                self.settings_panel()
                    .load_format(self.canvas().sheet().format());
                self.settings_panel()
                    .load_background(self.canvas().sheet().background().borrow_mut().clone());

                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );
                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().selection().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );

                self.canvas().queue_resize();
                self.canvas().queue_draw();
                self.canvas().set_unsaved_changes(false);

                Ok(())
            }
            utils::FileType::Svg => {
                let pos = if let Some(vadjustment) = self.canvas_scroller().vadjustment() {
                    na::vector![
                        VectorImage::OFFSET_X_DEFAULT,
                        vadjustment.value() + VectorImage::OFFSET_Y_DEFAULT
                    ]
                } else {
                    na::vector![VectorImage::OFFSET_X_DEFAULT, VectorImage::OFFSET_Y_DEFAULT]
                };
                self.canvas().sheet().import_file_as_svg(pos, file)?;

                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );
                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().selection().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );

                self.canvas()
                    .sheet()
                    .selection()
                    .emit_by_name("redraw", &[])
                    .unwrap();
                self.canvas().queue_draw();

                self.canvas().set_unsaved_changes(true);
                self.mainheader().selector_toggle().set_active(true);
                Ok(())
            }
            utils::FileType::BitmapImage => {
                let pos = if let Some(vadjustment) = self.canvas_scroller().vadjustment() {
                    na::vector![
                        BitmapImage::OFFSET_X_DEFAULT,
                        vadjustment.value() + BitmapImage::OFFSET_Y_DEFAULT
                    ]
                } else {
                    na::vector![BitmapImage::OFFSET_X_DEFAULT, BitmapImage::OFFSET_Y_DEFAULT]
                };
                self.canvas()
                    .sheet()
                    .import_file_as_bitmapimage(pos, file)?;

                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );
                StrokeStyle::update_all_rendernodes(
                    &mut *self.canvas().sheet().selection().strokes().borrow_mut(),
                    self.canvas().scalefactor(),
                    &*self.canvas().renderer().borrow(),
                );

                self.canvas()
                    .sheet()
                    .selection()
                    .emit_by_name("redraw", &[])
                    .unwrap();
                self.canvas().queue_draw();

                self.canvas().set_unsaved_changes(true);
                self.mainheader().selector_toggle().set_active(true);

                Ok(())
            }
            utils::FileType::Folder | utils::FileType::Unknown => {
                log::warn!("tried to open unsupported file type.");
                Ok(())
            }
        }
    }
}
