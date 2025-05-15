use crate::state::AppState; // Corrected typo here
use crate::ui;
use crate::logic;
use eframe::{egui, App, CreationContext};

pub struct MyApp {
    state: AppState,
    initialized: bool,
}

impl MyApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {


        // Set the visual style to light mode upon creation
        Self {
            
            state: AppState::default(), // Initializes with CNC defaults
            initialized: false,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.initialized {
            logic::perform_initial_setup(ctx, &mut self.state);
            self.initialized = true;
        }

        ui::draw_main_ui(ctx, &mut self.state);

        // Repaint if simulation or profile loading is in a "pending" state
        let sim_status_lower = self.state.simulation_status.to_lowercase();
        let profile_status_lower = self.state.profile_load_status.to_lowercase();

        if sim_status_lower.contains("simulating") ||
           sim_status_lower.contains("processing") ||
           profile_status_lower.contains("generating") ||
           profile_status_lower.contains("loading") {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

