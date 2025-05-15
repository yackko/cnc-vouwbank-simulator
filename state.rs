// src/state.rs

use egui::{TextureHandle, Vec2, Color32}; // Added Color32
use std::collections::HashMap; // For materials map

// --- CNC Specific Enums and Structs ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)] // Added Hash for HashMap key
pub enum MaterialName {
    Steel,
    Aluminum,
    StainlessSteel,
    Copper,
    MildSteel,
    Custom(String), // Allow custom material names
}

impl MaterialName {
    pub fn default_names() -> Vec<Self> {
        vec![
            MaterialName::Steel,
            MaterialName::Aluminum,
            MaterialName::StainlessSteel,
            MaterialName::Copper,
            MaterialName::MildSteel,
        ]
    }

    pub fn to_string(&self) -> String {
        match self {
            MaterialName::Steel => "Steel".to_string(),
            MaterialName::Aluminum => "Aluminum".to_string(),
            MaterialName::StainlessSteel => "Stainless Steel".to_string(),
            MaterialName::Copper => "Copper".to_string(),
            MaterialName::MildSteel => "Mild Steel".to_string(),
            MaterialName::Custom(name) => name.clone(),
        }
    }
    // You might want a FromStr implementation too for parsing user input
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialDetails {
    pub name: MaterialName,
    pub density_kg_m3: f64,    // kg/m^3
    pub yield_stress_mpa: f64, // MPa
    pub tensile_modulus_gpa: f64, // GPa (Young's Modulus)
    pub min_bend_radius_factor: f64, // Factor times thickness
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BendDirection {
    Up,
    Down,
}

impl BendDirection {
    pub fn default_directions() -> Vec<Self> {
        vec![BendDirection::Up, BendDirection::Down]
    }
    pub fn to_string(&self) -> String {
        match self {
            BendDirection::Up => "Up".to_string(),
            BendDirection::Down => "Down".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BendStep {
    pub sequence_order: usize, // 1-based
    pub position_mm: f64,
    pub target_angle_deg: f64,
    pub radius_mm: f64,
    pub direction: BendDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SheetMetal {
    pub id: String,
    pub original_length_mm: f64,
    pub thickness_mm: f64,
    pub width_mm: f64,
    pub material_name: MaterialName, // Store by name, lookup details in AppState.materials
    // CurrentBends would be part of a "ProcessedSheet" or similar,
    // or the simulation would directly modify a visual representation.
    // For now, let's assume the BendSteps in the Job define the target.
}

impl Default for SheetMetal {
    fn default() -> Self {
        SheetMetal {
            id: "DefaultSheet-001".to_string(),
            original_length_mm: 300.0,
            thickness_mm: 2.0,
            width_mm: 100.0,
            material_name: MaterialName::Steel,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Punch {
    pub name: String,
    pub height_mm: f64,
    pub angle_deg: f64,
    pub radius_mm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Die {
    pub name: String,
    pub v_opening_mm: f64,
    pub angle_deg: f64,
    pub shoulder_radius_mm: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub name: String,
    pub sheet: SheetMetal, // The workpiece definition for this job
    pub steps: Vec<BendStep>,
}

impl Default for Job {
    fn default() -> Self {
        Job {
            name: "DefaultJob-001".to_string(),
            sheet: SheetMetal::default(),
            steps: Vec::new(),
        }
    }
}

// --- UI Input State ---
#[derive(Default, Clone)]
pub struct SheetInputState {
    pub length_mm_str: String,
    pub thickness_mm_str: String,
    pub width_mm_str: String,
    pub selected_material_idx: usize,
}

#[derive(Default, Clone)]
pub struct BendInputState {
    pub position_mm_str: String,
    pub target_angle_deg_str: String,
    pub radius_mm_str: String,
    pub selected_direction_idx: usize,
}

#[derive(Default, Clone)]
pub struct ToolingInputState {
    pub selected_punch_idx: usize,
    pub selected_die_idx: usize,
}


// --- Main Application State ---
pub struct AppState {
    // Core Data
    pub current_job: Job,
    pub available_materials: HashMap<MaterialName, MaterialDetails>,
    pub material_display_order: Vec<MaterialName>, // For consistent UI dropdown order

    pub available_punches: Vec<Punch>,
    pub available_dies: Vec<Die>,

    // UI Interaction State
    pub sheet_input: SheetInputState,
    pub bend_input: BendInputState,
    pub tooling_input: ToolingInputState,

    // Simulation / Output State
    pub simulation_status: String, // e.g., "Ready", "Processing...", "Bend 1/5 complete"
    pub parts_bent_session: u32,
    pub simulated_profile_texture: Option<TextureHandle>, // For the SVG or rendered profile
    pub simulated_profile_size: Option<Vec2>,
    pub profile_load_status: String,

    // General UI state
    pub status_message: (String, Option<Color32>), // Message and optional color (e.g. for errors)
    // ... any other general state from the previous skeleton, like image_texture for a logo
    pub app_logo_texture: Option<TextureHandle>,
    pub app_logo_size: Option<Vec2>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut materials = HashMap::new();
        materials.insert(MaterialName::Steel, MaterialDetails { name: MaterialName::Steel, density_kg_m3: 7850.0, yield_stress_mpa: 250.0, tensile_modulus_gpa: 200.0, min_bend_radius_factor: 1.5 });
        materials.insert(MaterialName::Aluminum, MaterialDetails { name: MaterialName::Aluminum, density_kg_m3: 2700.0, yield_stress_mpa: 100.0, tensile_modulus_gpa: 70.0, min_bend_radius_factor: 1.0 });
        materials.insert(MaterialName::StainlessSteel, MaterialDetails { name: MaterialName::StainlessSteel, density_kg_m3: 8000.0, yield_stress_mpa: 215.0, tensile_modulus_gpa: 193.0, min_bend_radius_factor: 2.0 });
        materials.insert(MaterialName::Copper, MaterialDetails { name: MaterialName::Copper, density_kg_m3: 8960.0, yield_stress_mpa: 70.0, tensile_modulus_gpa: 117.0, min_bend_radius_factor: 0.8 });
        materials.insert(MaterialName::MildSteel, MaterialDetails { name: MaterialName::MildSteel, density_kg_m3: 7850.0, yield_stress_mpa: 220.0, tensile_modulus_gpa: 200.0, min_bend_radius_factor: 1.2 });

        let material_display_order = MaterialName::default_names();

        let punches = vec![
            Punch { name: "P88.10.R06".to_string(), height_mm: 60.0, angle_deg: 88.0, radius_mm: 0.6 },
            Punch { name: "P30.15.R1".to_string(), height_mm: 65.0, angle_deg: 30.0, radius_mm: 1.0 },
            Punch { name: "Default Punch".to_string(), height_mm: 50.0, angle_deg: 90.0, radius_mm: 1.0 },
        ];
        let dies = vec![
            Die { name: "D12.90.R2".to_string(), v_opening_mm: 12.0, angle_deg: 90.0, shoulder_radius_mm: 2.0 },
            Die { name: "D20.60.R3".to_string(), v_opening_mm: 20.0, angle_deg: 60.0, shoulder_radius_mm: 3.0 },
            Die { name: "Default Die".to_string(), v_opening_mm: 16.0, angle_deg: 90.0, shoulder_radius_mm: 2.0 },
        ];

        let current_job = Job::default();
        let mut sheet_input = SheetInputState {
            length_mm_str: current_job.sheet.original_length_mm.to_string(),
            thickness_mm_str: current_job.sheet.thickness_mm.to_string(),
            width_mm_str: current_job.sheet.width_mm.to_string(),
            selected_material_idx: material_display_order.iter().position(|n| *n == current_job.sheet.material_name).unwrap_or(0),
        };


        Self {
            current_job,
            available_materials: materials,
            material_display_order,
            available_punches: punches,
            available_dies: dies,
            sheet_input,
            bend_input: BendInputState::default(),
            tooling_input: ToolingInputState::default(),
            simulation_status: "Ready".to_string(),
            parts_bent_session: 0,
            simulated_profile_texture: None,
            simulated_profile_size: None,
            profile_load_status: "Profile not generated.".to_string(),
            status_message: ("System Initialized.".to_string(), None),
            app_logo_texture: None,
            app_logo_size: None,
        }
    }
}


