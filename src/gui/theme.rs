use egui::{Context, FontData, FontDefinitions, FontFamily, Visuals};

pub fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "THSarabunNew".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/THSarabunNew.ttf")).into(),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "THSarabunNew".to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "THSarabunNew".to_owned());

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &Context, dark_mode: bool) {
    if dark_mode {
        ctx.set_visuals(Visuals::dark());
    } else {
        ctx.set_visuals(Visuals::light());
    }
}
