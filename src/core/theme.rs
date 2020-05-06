use nalgebra_glm::{Vec2, Vec4};

pub struct Theme {
    // Main theme settings
    alpha: f32,
    anti_aliased_fill: bool,
    anti_aliased_lines: bool,
    child_rounding: f32,
    frame_padding: Vec2,
    frame_rounding: f32,
    grab_rounding: f32,
    indent_spacing: f32,
    item_spacing: Vec2,
    scrollbar_rounding: f32,
    scrollbar_size: f32,
    window_rounding: f32,

    // Colors
    border_shadow: Vec4,
    border: Vec4,
    button_active: Vec4,
    button_hovered: Vec4,
    button: Vec4,
    check_mark: Vec4,
    child_background: Vec4,
    drag_drop_target: Vec4,
    frame_background_active: Vec4,
    frame_background_hovered: Vec4,
    frame_background: Vec4,
    header_active: Vec4,
    header_hovered: Vec4,
    header: Vec4,
    menu_bar_background: Vec4,
    modal_window_dim_background: Vec4,
    nav_highlight: Vec4,
    nav_windowing_highlight: Vec4,
    plot_histogram_hovered: Vec4,
    plot_histogram: Vec4,
    plot_lines_hovered: Vec4,
    plot_lines: Vec4,
    popup_background: Vec4,
    resize_grip_active: Vec4,
    resize_grip_hovered: Vec4,
    resize_grip: Vec4,
    scrollbar_background: Vec4,
    scrollbar_grab_active: Vec4,
    scrollbar_grab_hovered: Vec4,
    scrollbar_grab: Vec4,
    separator_active: Vec4,
    separator_hovered: Vec4,
    separator: Vec4,
    slider_grab_active: Vec4,
    slider_grab: Vec4,
    text_disabled: Vec4,
    text_selected_background: Vec4,
    text: Vec4, 
    title_background_active: Vec4,
    title_background_collapsed: Vec4,
    title_background: Vec4,
    window_background: Vec4,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Main theme settings
            alpha: 1.0,
            anti_aliased_fill:  true,
            anti_aliased_lines:  true,
            child_rounding: 2.0,
            frame_padding:  Vec2::new(6.0, 4.0),
            frame_rounding: 3.0,
            grab_rounding: 2.0,
            indent_spacing: 22.0,
            item_spacing:  Vec2::new(10.0, 4.0),
            scrollbar_rounding: 3.0,
            scrollbar_size: 16.0,
            window_rounding: 2.0,
            // Colors
            border_shadow: Vec4::new(0.00, 0.00, 0.00, 0.04),
            border: Vec4::new(0.71, 0.71, 0.71, 0.08),
            button_active: Vec4::new(0.793, 0.900, 0.836, 1.00),
            button_hovered: Vec4::new(0.725, 0.805, 0.702, 1.00),
            button: Vec4::new(0.71, 0.78, 0.69, 0.40),
            check_mark: Vec4::new(0.184, 0.407, 0.193, 1.00),
            child_background: Vec4::new(0.00, 0.00, 0.00, 0.00),
            drag_drop_target: Vec4::new(0.26, 0.59, 0.98, 0.95),
            frame_background_active: Vec4::new(0.71, 0.78, 0.69, 0.98),
            frame_background_hovered: Vec4::new(0.94, 0.94, 0.94, 0.55),
            frame_background: Vec4::new(0.71, 0.71, 0.71, 0.55),
            header_active: Vec4::new(0.71, 0.78, 0.69, 1.00),
            header_hovered: Vec4::new(0.71, 0.78, 0.69, 0.80),
            header: Vec4::new(0.71, 0.78, 0.69, 0.31),
            menu_bar_background: Vec4::new(0.86, 0.86, 0.86, 1.00),
            modal_window_dim_background: Vec4::new(0.20, 0.20, 0.20, 0.35),
            nav_highlight: Vec4::new(0.71, 0.78, 0.69, 0.80),
            nav_windowing_highlight: Vec4::new(0.70, 0.70, 0.70, 0.70),
            plot_histogram_hovered: Vec4::new(1.00, 0.60, 0.00, 1.00),
            plot_histogram: Vec4::new(0.90, 0.70, 0.00, 1.00),
            plot_lines_hovered: Vec4::new(1.00, 0.43, 0.35, 1.00),
            plot_lines: Vec4::new(0.39, 0.39, 0.39, 1.00),
            popup_background: Vec4::new(0.93, 0.93, 0.93, 0.98),
            resize_grip_active: Vec4::new(0.26, 0.59, 0.98, 0.78),
            resize_grip_hovered: Vec4::new(0.26, 0.59, 0.98, 0.45),
            resize_grip: Vec4::new(1.00, 1.00, 1.00, 0.00),
            scrollbar_background: Vec4::new(0.20, 0.25, 0.30, 0.61),
            scrollbar_grab_active: Vec4::new(1.00, 1.00, 1.00, 1.00),
            scrollbar_grab_hovered: Vec4::new(0.92, 0.92, 0.92, 0.78),
            scrollbar_grab: Vec4::new(0.90, 0.90, 0.90, 0.30),
            separator_active: Vec4::new(0.26, 0.59, 0.98, 1.00),
            separator_hovered: Vec4::new(0.26, 0.59, 0.98, 0.78),
            separator: Vec4::new(0.39, 0.39, 0.39, 1.00),
            slider_grab_active: Vec4::new(0.26, 0.59, 0.98, 1.00),
            slider_grab: Vec4::new(0.26, 0.59, 0.98, 0.78),
            text_disabled: Vec4::new(0.60, 0.60, 0.60, 1.00),
            text_selected_background: Vec4::new(0.26, 0.59, 0.98, 0.35),
            text: Vec4::new(0.00, 0.00, 0.00, 1.00),
            title_background_active: Vec4::new(0.78, 0.78, 0.78, 1.00),
            title_background_collapsed: Vec4::new(0.82, 0.78, 0.78, 0.51),
            title_background: Vec4::new(0.85, 0.85, 0.85, 1.00),
            window_background: Vec4::new(0.86, 0.86, 0.86, 1.00),
        }
    }
}

impl Theme {
    pub fn update_imgui(&self, style: &mut imgui::Style) {
        let colors = &mut style.colors;

        style.alpha              = self.alpha;
        style.anti_aliased_fill  = self.anti_aliased_fill;
        style.anti_aliased_lines = self.anti_aliased_lines;
        style.child_rounding     = self.child_rounding;
        style.frame_padding[0]   = self.frame_padding.x;
        style.frame_padding[1]   = self.frame_padding.x;
        style.frame_rounding     = self.frame_rounding;
        style.grab_rounding      = self.grab_rounding;             // Radius of grabs corners rounding. Set to 0.0f to have rectangular slider grabs.
        style.indent_spacing     = self.indent_spacing;
        style.item_spacing[0]    = self.item_spacing.x;
        style.item_spacing[1]    = self.item_spacing.y;
        style.scrollbar_rounding = self.scrollbar_rounding;             // Radius of grab corners rounding for scrollbar
        style.scrollbar_size     = self.scrollbar_size;
        style.window_rounding    = self.window_rounding;             // Radius of window corners rounding. Set to 0.0f to have rectangular windows

        colors[imgui::StyleColor::Border as usize]                = self.border_shadow.into();
        colors[imgui::StyleColor::BorderShadow as usize]          = self.border.into();
        colors[imgui::StyleColor::Button as usize]                = self.button_active.into();
        colors[imgui::StyleColor::ButtonActive as usize]          = self.button_hovered.into();
        colors[imgui::StyleColor::ButtonHovered as usize]         = self.button.into();
        colors[imgui::StyleColor::CheckMark as usize]             = self.check_mark.into();
        colors[imgui::StyleColor::ChildBg as usize]               = self.child_background.into();
        colors[imgui::StyleColor::DragDropTarget as usize]        = self.drag_drop_target.into();
        colors[imgui::StyleColor::FrameBg as usize]               = self.frame_background_active.into();
        colors[imgui::StyleColor::FrameBgActive as usize]         = self.frame_background_hovered.into();
        colors[imgui::StyleColor::FrameBgHovered as usize]        = self.frame_background.into();
        colors[imgui::StyleColor::Header as usize]                = self.header_active.into();
        colors[imgui::StyleColor::HeaderActive as usize]          = self.header_hovered.into();
        colors[imgui::StyleColor::HeaderHovered as usize]         = self.header.into();
        colors[imgui::StyleColor::MenuBarBg as usize]             = self.menu_bar_background.into();
        colors[imgui::StyleColor::ModalWindowDimBg as usize]      = self.modal_window_dim_background.into();
        colors[imgui::StyleColor::NavHighlight as usize]          = self.nav_highlight.into();
        colors[imgui::StyleColor::NavWindowingHighlight as usize] = self.nav_windowing_highlight.into();
        colors[imgui::StyleColor::PlotHistogram as usize]         = self.plot_histogram_hovered.into();
        colors[imgui::StyleColor::PlotHistogramHovered as usize]  = self.plot_histogram.into();
        colors[imgui::StyleColor::PlotLines as usize]             = self.plot_lines_hovered.into();
        colors[imgui::StyleColor::PlotLinesHovered as usize]      = self.plot_lines.into();
        colors[imgui::StyleColor::PopupBg as usize]               = self.popup_background.into();
        colors[imgui::StyleColor::ResizeGrip as usize]            = self.resize_grip_active.into();
        colors[imgui::StyleColor::ResizeGripActive as usize]      = self.resize_grip_hovered.into();
        colors[imgui::StyleColor::ResizeGripHovered as usize]     = self.resize_grip.into();
        colors[imgui::StyleColor::ScrollbarBg as usize]           = self.scrollbar_background.into();
        colors[imgui::StyleColor::ScrollbarGrab as usize]         = self.scrollbar_grab_active.into();
        colors[imgui::StyleColor::ScrollbarGrabActive as usize]   = self.scrollbar_grab_hovered.into();
        colors[imgui::StyleColor::ScrollbarGrabHovered as usize]  = self.scrollbar_grab.into();
        colors[imgui::StyleColor::Separator as usize]             = self.separator_active.into();
        colors[imgui::StyleColor::SeparatorActive as usize]       = self.separator_hovered.into();
        colors[imgui::StyleColor::SeparatorHovered as usize]      = self.separator.into();
        colors[imgui::StyleColor::SliderGrab as usize]            = self.slider_grab_active.into();
        colors[imgui::StyleColor::SliderGrabActive as usize]      = self.slider_grab.into();
        colors[imgui::StyleColor::Text as usize]                  = self.text_disabled.into();
        colors[imgui::StyleColor::TextDisabled as usize]          = self.text_selected_background.into();
        colors[imgui::StyleColor::TextSelectedBg as usize]        = self.text.into();
        colors[imgui::StyleColor::TitleBg as usize]               = self.title_background_active.into();
        colors[imgui::StyleColor::TitleBgActive as usize]         = self.title_background_collapsed.into();
        colors[imgui::StyleColor::TitleBgCollapsed as usize]      = self.title_background.into();
        colors[imgui::StyleColor::WindowBg as usize]              = self.window_background.into();
    }
}
