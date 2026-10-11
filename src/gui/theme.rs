use egui::{
    Color32, Context, FontFamily, FontId, Stroke, TextStyle, Theme, Visuals,
};

/// ฟอนต์ภาษาไทย TH Sarabun New ฝังตัวลงใน Binary
pub const THAI_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/THSarabunNew.ttf");

/// ติดตั้งฟอนต์ภาษาไทยและกำหนดขนาดการแสดงผลให้เหมาะสม
pub fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 1. ลงทะเบียนไฟล์ฟอนต์ภาษาไทย THSarabunNew
    fonts.font_data.insert(
        "THSarabunNew".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(THAI_FONT_BYTES)),
    );

    // 2. จัดลำดับ Priority 0 ให้กับ THSarabunNew ในทั้ง Proportional และ Monospace
    if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
        family.insert(0, "THSarabunNew".to_owned());
    }
    if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
        family.insert(0, "THSarabunNew".to_owned());
    }

    ctx.set_fonts(fonts);

    // 3. กำหนดขนาดฟอนต์เริ่มต้น (Font Sizes: 16.0 - 24.0 pt) ให้อ่านง่ายสำหรับภาษาไทย (ทั้ง Dark & Light theme)
    for theme in [Theme::Dark, Theme::Light] {
        ctx.style_mut_of(theme, |style| {
            style.text_styles = [
                (
                    TextStyle::Heading,
                    FontId::new(24.0, FontFamily::Proportional),
                ),
                (
                    TextStyle::Name("Subheading".into()),
                    FontId::new(20.0, FontFamily::Proportional),
                ),
                (TextStyle::Body, FontId::new(17.0, FontFamily::Proportional)),
                (
                    TextStyle::Monospace,
                    FontId::new(16.0, FontFamily::Monospace),
                ),
                (
                    TextStyle::Button,
                    FontId::new(17.0, FontFamily::Proportional),
                ),
                (
                    TextStyle::Small,
                    FontId::new(14.0, FontFamily::Proportional),
                ),
            ]
            .into();
        });
    }
}

/// ปรับแต่ง UI Visuals และ Theme (Dark / Light mode, Spacing, Padding) ให้ดูสะอาดตา สไตล์ Modern Engineering Dashboard
pub fn apply_theme(ctx: &Context, dark_mode: bool) {
    let theme = if dark_mode { Theme::Dark } else { Theme::Light };
    ctx.set_theme(theme);

    let mut visuals = if dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    if dark_mode {
        // Engineering Dark Theme Palette
        visuals.panel_fill = Color32::from_rgb(18, 22, 28);
        visuals.window_fill = Color32::from_rgb(24, 29, 37);
        visuals.faint_bg_color = Color32::from_rgb(30, 36, 46);
        visuals.extreme_bg_color = Color32::from_rgb(12, 15, 20);

        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(26, 32, 42);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(45, 55, 72));
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(226, 232, 240));

        visuals.widgets.inactive.bg_fill = Color32::from_rgb(33, 41, 54);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(203, 213, 225));

        visuals.widgets.hovered.bg_fill = Color32::from_rgb(44, 55, 72);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::from_rgb(255, 255, 255));

        visuals.widgets.active.bg_fill = Color32::from_rgb(14, 116, 144); // Cyan/Teal accent
        visuals.widgets.active.fg_stroke = Stroke::new(1.5, Color32::from_rgb(255, 255, 255));

        visuals.selection.bg_fill = Color32::from_rgb(14, 116, 144);
        visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(56, 189, 248));
    } else {
        // Clean Engineering Light Theme Palette
        visuals.panel_fill = Color32::from_rgb(248, 250, 252);
        visuals.window_fill = Color32::from_rgb(255, 255, 255);
        visuals.faint_bg_color = Color32::from_rgb(241, 245, 249);
        visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);

        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(226, 232, 240));
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(30, 41, 59));

        visuals.widgets.inactive.bg_fill = Color32::from_rgb(241, 245, 249);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(51, 65, 85));

        visuals.widgets.hovered.bg_fill = Color32::from_rgb(226, 232, 240);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::from_rgb(15, 23, 42));

        visuals.widgets.active.bg_fill = Color32::from_rgb(2, 132, 199);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5, Color32::from_rgb(255, 255, 255));

        visuals.selection.bg_fill = Color32::from_rgb(2, 132, 199);
        visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(14, 165, 233));
    }

    ctx.set_visuals_of(theme, visuals);

    // กำหนด Spacing, Item Spacing และ Padding สำหรับ Theme ที่เลือก
    ctx.style_mut_of(theme, |style| {
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(12);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
    });
}
