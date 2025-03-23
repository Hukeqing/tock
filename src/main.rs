use crate::render::{MonoRender, Render};
use crate::source::{Source, SourceManager};
use config::{Config, FileFormat};
use crossterm::terminal;

mod common;
mod render;
mod source;

#[tokio::main]
async fn main() {
    let config_file = Config::builder()
        .add_source(config::File::new("config.yaml", FileFormat::Yaml))
        .build()
        .unwrap();

    let mut setting = config_file.try_deserialize().unwrap();
    let mut manager = SourceManager::new(&mut setting).await;

    let mut width = 0usize;
    let mut height = 0usize;
    match terminal::size() {
        Ok((cols, rows)) => {
            width = cols as usize;
            height = rows as usize;
        }
        Err(e) => {
            panic!("Could not get terminal size: {}", e);
        }
    }

    let mut render = MonoRender::init(width, height);
    render
        .render(width, height, Vec::new(), "")
        .expect("TODO: panic message");

    while let Some(stock) = manager.recv().await {
        render.refresh_stock(stock);
    }

    // let mut stdout = stdout();
    // execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    //
    // let mut input = String::new();
    //
    // terminal::enable_raw_mode().unwrap();
    // loop {
    //     execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
    //     print!("请输入内容 (按回车确认, Esc 退出): \n\r{}", input);
    //     execute!(stdout, cursor::MoveToColumn(input.len() as u16)).unwrap();
    //
    //     {
    //         if event::poll(std::time::Duration::from_millis(500)).unwrap() {
    //             if let event::Event::Key(key_event) = event::read().unwrap() {
    //                 match key_event.code {
    //                     KeyCode::Esc => break, // 退出
    //                     KeyCode::Enter => {
    //                         println!("\n\r你输入了: {}", input);
    //                         input.clear(); // 清空输入
    //                     }
    //                     KeyCode::Char(c) => input.push(c), // 追加字符
    //                     KeyCode::Backspace => { input.pop(); } // 删除字符
    //                     _ => {}
    //                 }
    //             }
    //         }
    //     };
    // }
}
