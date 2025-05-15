use crate::state::{AppState, BendDirection, MaterialName}; // MaterialName is used for to_string
use crate::logic;
use egui::{Context, Ui, RichText, Color32, ComboBox, ScrollArea, TextEdit, Vec2}; // Color32 is used for status_message

fn sheet_properties_panel(ui: &mut Ui, state: &mut AppState) {
    ui.strong("Plaat Eigenschappen");
    ui.group(|ui| {
        egui::Grid::new("sheet_properties_grid")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Lengte (mm):");
                ui.add(TextEdit::singleline(&mut state.sheet_input.length_mm_str).desired_width(100.0));
                ui.end_row();

                ui.label("Dikte (mm):");
                ui.add(TextEdit::singleline(&mut state.sheet_input.thickness_mm_str).desired_width(100.0));
                ui.end_row();

                ui.label("Breedte (mm):");
                ui.add(TextEdit::singleline(&mut state.sheet_input.width_mm_str).desired_width(100.0));
                ui.end_row();

                ui.label("Materiaal:");
                ComboBox::from_id_source("material_select_cnc")
                    .selected_text(
                        state.material_display_order
                            .get(state.sheet_input.selected_material_idx)
                            .map_or_else(|| "N/A".to_string(), |m| m.to_string())
                    )
                    .width(150.0)
                    .show_index(
                        ui,
                        &mut state.sheet_input.selected_material_idx,
                        state.material_display_order.len(),
                        |i| state.material_display_order[i].to_string()
                    );
                ui.end_row();
            });
        
        ui.add_space(5.0);
        if ui.button("Update Plaat Eigenschappen").clicked() {
            logic::update_sheet_properties(state);
        }
        if let Some(min_rad) = logic::get_recommended_min_bend_radius(state) {
            ui.label(RichText::new(format!("Recommended Min Bend Radius: {:.2} mm", min_rad)).small());
        }
    });
}

fn tooling_setup_panel(ui: &mut Ui, state: &mut AppState) {
    ui.strong("Tooling Setup");
     ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Punch:");
            ComboBox::from_id_source("punch_select_cnc")
                .selected_text(
                    state.available_punches
                        .get(state.tooling_input.selected_punch_idx)
                        .map_or_else(|| "N/A".to_string(), |p| p.name.clone())
                )
                .width(150.0)
                .show_index(
                    ui,
                    &mut state.tooling_input.selected_punch_idx,
                    state.available_punches.len(),
                    |i| state.available_punches[i].name.clone()
                );
        });
        ui.horizontal(|ui|{
            ui.label("Die:    "); // Padding for alignment
            ComboBox::from_id_source("die_select_cnc")
                .selected_text(
                    state.available_dies
                        .get(state.tooling_input.selected_die_idx)
                        .map_or_else(|| "N/A".to_string(), |d| d.name.clone())
                )
                .width(150.0)
                .show_index(
                    ui,
                    &mut state.tooling_input.selected_die_idx,
                    state.available_dies.len(),
                    |i| state.available_dies[i].name.clone()
                );
        });
        ui.add_space(5.0);
        if let Some(punch) = state.available_punches.get(state.tooling_input.selected_punch_idx) {
            ui.label(RichText::new(format!("Selected Punch: {} (Angle: {}°, Radius: {}mm)", punch.name, punch.angle_deg, punch.radius_mm)).small());
        }
        if let Some(die) = state.available_dies.get(state.tooling_input.selected_die_idx) {
            ui.label(RichText::new(format!("Selected Die: {} (V-Open: {}mm, Angle: {}°)", die.name, die.v_opening_mm, die.angle_deg)).small());
        }
    });
}

fn bend_definition_panel(ui: &mut Ui, state: &mut AppState) {
    ui.strong("Defieër Buig Stap");
    ui.group(|ui| {
         egui::Grid::new("bend_def_grid_cnc")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Positie (mm):");
                ui.add(TextEdit::singleline(&mut state.bend_input.position_mm_str).desired_width(100.0));
                ui.end_row();

                ui.label("Gewenste Hoek (°):");
                ui.add(TextEdit::singleline(&mut state.bend_input.target_angle_deg_str).desired_width(100.0));
                ui.end_row();

                ui.label("Binnen Straal (mm):");
                ui.add(TextEdit::singleline(&mut state.bend_input.radius_mm_str).desired_width(100.0));
                ui.end_row();

                ui.label("Richting:");
                 ComboBox::from_id_source("bend_direction_select_cnc")
                    .selected_text(
                        BendDirection::default_directions()
                            .get(state.bend_input.selected_direction_idx)
                            .map_or_else(|| "N/A".to_string(), |d| d.to_string())
                    )
                    .width(100.0)
                    .show_index(
                        ui,
                        &mut state.bend_input.selected_direction_idx,
                        BendDirection::default_directions().len(),
                        |i| BendDirection::default_directions()[i].to_string()
                    );
                ui.end_row();
            });
        ui.add_space(5.0);
        if ui.button("Voeg Buiging Toe Aan De Job").clicked() {
            logic::add_bend_step(state);
        }
    });
}

fn bend_sequence_panel(ui: &mut Ui, state: &mut AppState) {
    ui.strong(format!("Huidge Job Buig Sequentie ({})", state.current_job.steps.len()));
    ui.group(|ui| {
        ScrollArea::vertical().max_height(150.0).min_scrolled_height(100.0).show(ui, |ui| {
            if state.current_job.steps.is_empty() {
                ui.label("Geen buig stappen gedefinieërd voor de huidige job.");
            } else {
                egui::Grid::new("bend_sequence_grid_cnc")
                    .num_columns(5) // #, Pos, Angle, Radius, Dir
                    .spacing([5.0, 2.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("#").strong());
                        ui.label(RichText::new("Pos").strong());
                        ui.label(RichText::new("Hoek").strong());
                        ui.label(RichText::new("Graden").strong());
                        ui.label(RichText::new("Dir").strong());
                        ui.end_row();

                        for step in &state.current_job.steps {
                            ui.label(step.sequence_order.to_string());
                            ui.label(format!("{:.1}", step.position_mm));
                            ui.label(format!("{:.1}", step.target_angle_deg));
                            ui.label(format!("{:.1}", step.radius_mm));
                            ui.label(step.direction.to_string());
                            ui.end_row();
                        }
                    });
            }
        });
        ui.add_space(5.0);
        if ui.button("Wis Alle Plooi Stappen").clicked() {
            logic::clear_all_bend_steps(state);
        }
    });
}

fn execution_panel(ui: &mut Ui, state: &mut AppState, ctx: &Context) {
    ui.strong("Machine Bediening");
    ui.group(|ui| {
        if ui.button("Voer Simulatie Uit & Genereer Profiel").clicked() {
            logic::run_simulation(state, ctx);
        }
        ui.add_space(5.0);
        ui.label(format!("Machine Status: {}", state.simulation_status));
        ui.label(format!("Geplooide Onderdelen Deze Sessie: {}", state.parts_bent_session));
    });
}

fn profile_display_panel(ui: &mut Ui, state: &mut AppState) {
    ui.strong("Simulatie Profiel Plaat");
    ui.group(|ui| {
        let desired_height = ui.available_height().max(200.0);
        ui.allocate_ui(Vec2::new(ui.available_width(), desired_height), |ui_inner| {
            ui_inner.centered_and_justified(|ui_centered| {
                if let (Some(texture), Some(size_val)) = (&state.simulated_profile_texture, state.simulated_profile_size) {
                    // Corrected: No dereference needed for size_val as it's already Vec2
                    let mut display_size = size_val; 
                    let aspect_ratio = size_val.x / size_val.y;

                    if display_size.x > ui_centered.available_width() {
                        display_size.x = ui_centered.available_width();
                        display_size.y = display_size.x / aspect_ratio;
                    }
                    if display_size.y > ui_centered.available_height() {
                        display_size.y = ui_centered.available_height();
                        display_size.x = display_size.y * aspect_ratio;
                    }
                    ui_centered.image((texture.id(), display_size));
                } else {
                    ui_centered.label(&state.profile_load_status);
                }
            });
        });
    });
}

fn status_bar(ui: &mut Ui, state: &AppState) {
    let (text, color_opt) = &state.status_message; // Renamed color to color_opt for clarity
    let rich_text = if let Some(c) = color_opt { // c is &Color32
        RichText::new(text).color(*c) // Dereference c
    } else {
        RichText::new(text).color(ui.style().visuals.text_color())
    };
    ui.separator();
    ui.label(rich_text);
}

fn file_menu(ui: &mut Ui, state: &mut AppState, _ctx: &Context) {
    ui.menu_button("Bestand", |ui| {
        if ui.button("Laad Taak...").clicked() {
            logic::handle_load_job(state, Some("jobs/sample_job.json".to_string()));
            ui.close_menu();
        }
        if ui.button("Laad Taak Als...").clicked() {
            logic::handle_save_job(state, Some("jobs/my_output_job.json".to_string()));
            ui.close_menu();
        }
        if ui.button("Exit").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

pub fn draw_main_ui(ctx: &Context, state: &mut AppState) {
    egui::TopBottomPanel::top("menu_bar_panel_cnc").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            file_menu(ui, state, ctx);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let (Some(logo_tex), Some(logo_size_val)) = (&state.app_logo_texture, state.app_logo_size) {
                    let desired_height = ui.max_rect().height() * 0.8;
                    let aspect_ratio = logo_size_val.x / logo_size_val.y;
                    let desired_width = desired_height * aspect_ratio;
                    ui.image((logo_tex.id(), Vec2::new(desired_width, desired_height)));
                }
                ui.label(RichText::new(format!("CNC Plooibank Sim v0.1 ({})", chrono::Local::now().format("%H:%M:%S"))).small());
            });
        });
    });

    egui::SidePanel::left("input_controls_panel_cnc")
        .resizable(true)
        .default_width(380.0)
        .width_range(300.0..=500.0)
        .show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.heading("Taak & Machine Setup");
                ui.add_space(10.0);
                sheet_properties_panel(ui, state);
                ui.add_space(10.0);
                tooling_setup_panel(ui, state);
                ui.add_space(10.0);
                bend_definition_panel(ui, state);
                ui.add_space(10.0);
                bend_sequence_panel(ui, state);
                ui.add_space(10.0);
            });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Bediening & Uitvoer");
        ui.separator();
        execution_panel(ui, state, ctx);
        ui.separator();
        profile_display_panel(ui, state);
    });

    egui::TopBottomPanel::bottom("bottom_status_bar_panel_cnc").show(ctx, |ui| {
        status_bar(ui, state);
    });
}

