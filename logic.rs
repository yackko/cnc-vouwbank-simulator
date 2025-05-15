// src/logic.rs
use crate::state::{AppState, BendStep, BendDirection, SheetMetal, MaterialName, Job, SheetInputState, BendInputState, ToolingInputState};
use crate::db::{self, JobStorageError}; // Assuming db.rs is at this path
use egui::{Context, Color32, Vec2, ColorImage, TextureHandle};
use image::GenericImageView;
use std::path::Path;

// --- Config Constants (could be in a separate config.rs) ---
const MIN_SHEET_DIMENSION_MM: f64 = 0.1;
const MAX_SHEET_DIMENSION_MM: f64 = 10000.0;
const MIN_BEND_RADIUS_MM: f64 = 0.0; // 0 can mean sharp
const MAX_BEND_RADIUS_MM: f64 = 500.0;
const MIN_BEND_ANGLE_DEG: f64 = 1.0;
const MAX_BEND_ANGLE_DEG: f64 = 179.0;


// --- Image Logic (from previous skeleton, adapted) ---
#[derive(Debug, thiserror::Error)]
pub enum ImageLogicError {
    #[error("Failed to load image from path: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to decode image: {0}")]
    ImageError(#[from] image::ImageError),
}

fn load_image_and_create_texture(
    ctx: &Context,
    image_path_str: &str,
    texture_name: &str,
) -> Result<(TextureHandle, Vec2), ImageLogicError> {
    let image_data = std::fs::read(image_path_str)?;
    let image_dyn = image::load_from_memory(&image_data)?;
    let (width, height) = image_dyn.dimensions();
    let image_rgba = image_dyn.to_rgba8();

    let egui_image = ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        image_rgba.as_flat_samples().as_slice(),
    );

    let texture_handle = ctx.load_texture(
        texture_name, // Unique name for the texture
        egui_image,
        Default::default(),
    );
    Ok((texture_handle, Vec2::new(width as f32, height as f32)))
}


// --- CNC Specific Logic ---

pub fn update_sheet_properties(state: &mut AppState) {
    let parse_f64 = |s: &str, field_name: &str| -> Result<f64, String> {
        s.parse::<f64>().map_err(|_| format!("Invalid {}: '{}' is not a valid number.", field_name, s))
    };

    let length = match parse_f64(&state.sheet_input.length_mm_str, "Length") {
        Ok(l) if l >= MIN_SHEET_DIMENSION_MM && l <= MAX_SHEET_DIMENSION_MM => l,
        Ok(_) => {
            state.status_message = (format!("Length out of range ({}-{}mm).", MIN_SHEET_DIMENSION_MM, MAX_SHEET_DIMENSION_MM), Some(Color32::RED));
            return;
        }
        Err(e) => { state.status_message = (e, Some(Color32::RED)); return; }
    };
    // Similar parsing and validation for thickness and width
    let thickness = match parse_f64(&state.sheet_input.thickness_mm_str, "Thickness") {
        Ok(t) if t >= MIN_SHEET_DIMENSION_MM && t <= MAX_SHEET_DIMENSION_MM => t,
        Ok(_) => {
            state.status_message = (format!("Thickness out of range ({}-{}mm).", MIN_SHEET_DIMENSION_MM, MAX_SHEET_DIMENSION_MM), Some(Color32::RED));
            return;
        }
        Err(e) => { state.status_message = (e, Some(Color32::RED)); return; }
    };
     let width = match parse_f64(&state.sheet_input.width_mm_str, "Width") {
        Ok(w) if w >= MIN_SHEET_DIMENSION_MM && w <= MAX_SHEET_DIMENSION_MM => w,
        Ok(_) => {
            state.status_message = (format!("Width out of range ({}-{}mm).", MIN_SHEET_DIMENSION_MM, MAX_SHEET_DIMENSION_MM), Some(Color32::RED));
            return;
        }
        Err(e) => { state.status_message = (e, Some(Color32::RED)); return; }
    };


    let selected_material_name = state.material_display_order.get(state.sheet_input.selected_material_idx)
        .cloned()
        .unwrap_or_else(|| {
            // Fallback if index is somehow out of bounds, though UI should prevent this.
            // Or, if material_display_order is empty (which it shouldn't be from Default).
            state.material_display_order.first().cloned().unwrap_or(MaterialName::Steel)
        });


    state.current_job.sheet.original_length_mm = length;
    state.current_job.sheet.thickness_mm = thickness;
    state.current_job.sheet.width_mm = width;
    state.current_job.sheet.material_name = selected_material_name;
    state.current_job.steps.clear(); // Changing sheet properties invalidates old bends

    state.status_message = ("Sheet properties updated. Bend steps cleared.".to_string(), Some(Color32::GREEN));
    state.simulated_profile_texture = None; // Clear old profile
    state.profile_load_status = "Profile outdated due to sheet change.".to_string();
}

pub fn get_recommended_min_bend_radius(state: &AppState) -> Option<f64> {
    let sheet = &state.current_job.sheet;
    state.available_materials.get(&sheet.material_name).map(|details| {
        if sheet.thickness_mm <= 0.0 { return 0.0; }
        if details.min_bend_radius_factor <= 0.0 {
            sheet.thickness_mm * 0.5 // Default fallback
        } else {
            sheet.thickness_mm * details.min_bend_radius_factor
        }
    })
}


pub fn add_bend_step(state: &mut AppState) {
    let parse_f64 = |s: &str, field_name: &str| -> Result<f64, String> {
        s.parse::<f64>().map_err(|_| format!("Ongeldige {}: '{}'", field_name, s))
    };

    let position = match parse_f64(&state.bend_input.position_mm_str, "Buig Positie") {
        Ok(p) if p > 0.0 && p < state.current_job.sheet.original_length_mm => p,
        Ok(p) => {
            state.status_message = (format!("Buig posititie ({}mm) is buiten de plaat lengte (0-{}mm).", p, state.current_job.sheet.original_length_mm), Some(Color32::RED));
            return;
        }
        Err(e) => { state.status_message = (e, Some(Color32::RED)); return; }
    };
    // Similar parsing and validation for angle and radius
    let angle = match parse_f64(&state.bend_input.target_angle_deg_str, "Buig Hoek") {
        Ok(a) if a >= MIN_BEND_ANGLE_DEG && a <= MAX_BEND_ANGLE_DEG => a,
        Ok(a) => {
            state.status_message = (format!("Buig hoek ({}°) buiten bereik van ({}-{}°).", a, MIN_BEND_ANGLE_DEG, MAX_BEND_ANGLE_DEG), Some(Color32::RED));
            return;
        }
        Err(e) => { state.status_message = (e, Some(Color32::RED)); return; }
    };
    let radius = match parse_f64(&state.bend_input.radius_mm_str, "Buig Radius") {
        Ok(r) if r >= MIN_BEND_RADIUS_MM && r <= MAX_BEND_RADIUS_MM => r,
        Ok(r) => {
             state.status_message = (format!("Buig radius ({}mm) buiten bereik van ({}-{}mm).", r, MIN_BEND_RADIUS_MM, MAX_BEND_RADIUS_MM), Some(Color32::RED));
            return;
        }
        Err(e) => { state.status_message = (e, Some(Color32::RED)); return; }
    };

    let direction = BendDirection::default_directions()
        .get(state.bend_input.selected_direction_idx)
        .cloned()
        .unwrap_or(BendDirection::Up); // Fallback

    // Optional: Check against recommended min bend radius
    if let Some(min_recommended_radius) = get_recommended_min_bend_radius(state) {
        if radius > 1e-6 && radius < min_recommended_radius { // allow 0 for sharp/coining effectively
            // For now, just a log or status. A real app might use a dialog as in the Gio example.
            state.status_message = (format!("Warning: Radius {:.2}mm < recommended min {:.2}mm for material.", radius, min_recommended_radius), Some(Color32::YELLOW));
        }
    }


    let new_step = BendStep {
        sequence_order: state.current_job.steps.len() + 1,
        position_mm: position,
        target_angle_deg: angle,
        radius_mm: radius,
        direction,
    };
    state.current_job.steps.push(new_step);
    state.status_message = ("Buig stap toegevoegd.".to_string(), Some(Color32::GREEN));
    state.simulated_profile_texture = None; // Profile outdated
    state.profile_load_status = "Profile outdated due to new bend.".to_string();
}

pub fn clear_all_bend_steps(state: &mut AppState) {
    if state.current_job.steps.is_empty() {
        state.status_message = ("No bend steps to clear.".to_string(), None);
        return;
    }
    state.current_job.steps.clear();
    state.status_message = ("All bend steps cleared.".to_string(), Some(Color32::GREEN));
    state.simulated_profile_texture = None; // Profile outdated
    state.profile_load_status = "Profile outdated, bends cleared.".to_string();
}

pub fn run_simulation(state: &mut AppState, ctx: &Context) {
    if state.current_job.steps.is_empty() {
        state.status_message = ("No bend steps to simulate.".to_string(), Some(Color32::YELLOW));
        return;
    }
    // --- Actual simulation logic would go here ---
    // This might involve complex geometry calculations to determine the final shape.
    // For this skeleton, we'll just log it and generate a placeholder "profile".
    state.simulation_status = format!("Simulating {} bend steps for job '{}'...", state.current_job.steps.len(), state.current_job.name);
    state.status_message = (state.simulation_status.clone(), None);
    println!("{}", state.simulation_status);
    for step in &state.current_job.steps {
        println!("  Simulating Step {}: Pos: {}, Angle: {}, Rad: {}, Dir: {:?}",
            step.sequence_order, step.position_mm, step.target_angle_deg, step.radius_mm, step.direction);
    }

    // Simulate generating an SVG and loading it as a texture
    // In a real app, you'd call a proper SVG generation function.
    // For now, let's try to load the app_logo as a placeholder "profile".
    // This section should ideally generate an SVG based on `state.current_job`
    // then render that SVG to a `ColorImage` (e.g. using `resvg`), then load as texture.
    // That's quite involved, so we'll reuse the app logo loading logic for now.

    let temp_profile_path = "assets/drawing.png"; // Placeholder
    state.profile_load_status = format!("Generating profile (using placeholder: {})...", temp_profile_path);
    ctx.request_repaint(); // show status update

    match load_image_and_create_texture(ctx, temp_profile_path, "simulated_profile") {
        Ok((texture, size)) => {
            state.simulated_profile_texture = Some(texture);
            state.simulated_profile_size = Some(size);
            state.profile_load_status = "Simulated profile loaded (placeholder).".to_string();
        }
        Err(e) => {
            state.profile_load_status = format!("Fout laden profiel afbeelding: {}", e);
            state.simulated_profile_texture = None;
        }
    }

    state.parts_bent_session += 1;
    state.simulation_status = "Simulatie compleet.".to_string();
    state.status_message = ("Simulatie compleet.".to_string(), Some(Color32::GREEN));
}


pub fn perform_initial_setup(ctx: &Context, state: &mut AppState) {
    // Load app logo (example from previous skeleton)
    match load_image_and_create_texture(ctx, "assets/drawing.png", "app_logo") {
        Ok((texture, size)) => {
            state.app_logo_texture = Some(texture);
            state.app_logo_size = Some(size);
        }
        Err(e) => {
            eprintln!("Failed to load app logo: {}", e);
            // Update status message if you have one for app-level errors
            state.status_message = (format!("Failed to load app logo: {}", e), Some(Color32::RED));
        }
    }

    // You could load a default job here, or initialize other CNC specific things.
    state.sheet_input.length_mm_str = state.current_job.sheet.original_length_mm.to_string();
    state.sheet_input.thickness_mm_str = state.current_job.sheet.thickness_mm.to_string();
    state.sheet_input.width_mm_str = state.current_job.sheet.width_mm.to_string();
    state.sheet_input.selected_material_idx = state.material_display_order.iter().position(|n| *n == state.current_job.sheet.material_name).unwrap_or(0);


    // Populate default bend input values for convenience
    state.bend_input.position_mm_str = "50.0".to_string();
    state.bend_input.target_angle_deg_str = "90.0".to_string();
    state.bend_input.radius_mm_str = "2.0".to_string();
}

pub fn handle_save_job(state: &AppState, file_path: Option<String>) {
    if let Some(path) = file_path {
        match db::save_job_to_file(&state.current_job, &path) {
            Ok(_) => println!("Job '{}' saved to '{}'", state.current_job.name, path), // Update state.status_message
            Err(e) => eprintln!("Failed to save job: {}", e), // Update state.status_message
        }
    } else {
        println!("Save job cancelled."); // Update state.status_message
    }
}

pub fn handle_load_job(state: &mut AppState, file_path: Option<String>) {
    if let Some(path) = file_path {
        match db::load_job_from_file(&path) {
            Ok(loaded_job) => {
                state.current_job = loaded_job;
                // Update input fields from loaded job
                state.sheet_input.length_mm_str = state.current_job.sheet.original_length_mm.to_string();
                state.sheet_input.thickness_mm_str = state.current_job.sheet.thickness_mm.to_string();
                state.sheet_input.width_mm_str = state.current_job.sheet.width_mm.to_string();
                state.sheet_input.selected_material_idx = state.material_display_order.iter().position(|n| *n == state.current_job.sheet.material_name).unwrap_or(0);
                // Clear bend input fields or populate from first loaded bend? For now, clear.
                state.bend_input = BendInputState::default();
                state.simulated_profile_texture = None; // Clear old profile
                state.profile_load_status = "New job loaded, profile outdated.".to_string();
                println!("Job loaded from '{}'", path); // Update state.status_message
            }
            Err(e) => eprintln!("Failed to load job: {}", e), // Update state.status_message
        }
    } else {
        println!("Load job cancelled."); // Update state.status_message
    }
}



