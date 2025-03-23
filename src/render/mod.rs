use crate::common::Stock;
pub use crate::render::mono::MonoRender;

pub mod mono;

pub trait Render {
    /// init for render system
    fn init(width: usize, height: usize) -> Self where Self: Sized;

    /// render for all stock, when any stock is added or window resized
    fn render(&mut self, width: usize, height: usize, stocks: Vec<Stock>, command: &str) -> Result<(), String>;

    /// refresh a stock which has been added
    fn refresh_stock(&mut self, stock: Stock);

    /// refresh command line
    fn refresh_command(&mut self, command: &str);
}
