
pub mod audio;
pub mod state;
pub mod windowing;
pub mod emulator_loop;
pub mod app;
use app::Application;
fn main() {
    let mut app = Application::new();
    app.run();
}