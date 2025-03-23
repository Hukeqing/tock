use crate::common::Stock;
use crate::render::Render;
use crossterm::{cursor, execute, terminal::Clear, terminal::ClearType};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::io::{stdout, Stdout, Write};

pub struct MonoRender {
    page_no: usize,
    page_size: usize,
    stdout: Stdout,
    stock_index_map: HashMap<String, usize>,
    stocks: Vec<Stock>,
    renders: Vec<(usize, fn(Option<&Stock>, usize))>,
}

impl Render for MonoRender {
    fn init(_width: usize, height: usize) -> Self
    where
        Self: Sized,
    {
        // terminal::enable_raw_mode().unwrap();
        MonoRender {
            page_no: 1,
            page_size: height - 2,
            stdout: stdout(),
            stock_index_map: HashMap::new(),
            stocks: Vec::new(),
            renders: Vec::new(),
        }
    }

    fn render(
        &mut self,
        width: usize,
        height: usize,
        stocks: Vec<Stock>,
        command: &str,
    ) -> Result<(), String> {
        self.stocks = stocks;
        self.stock_index_map = HashMap::new();
        for i in 0..self.stocks.len() {
            self.stock_index_map
                .insert(self.stocks.get(i).unwrap().symbol.clone(), i);
        }

        self.build_render_map(width, height)?;
        execute!(self.stdout, Clear(ClearType::All)).unwrap();
        self.move_cursor(0, 0);
        for render in &self.renders {
            let (limit, func) = render;
            func(None, *limit);
        }
        for i in 0..self.page_size {
            let j = i + (self.page_no - 1) * self.page_size;
            self.refresh_stock_index(j);
        }
        self.refresh_command(command);
        Ok(())
    }

    fn refresh_stock(&mut self, stock: Stock) {
        let option = self.stock_index_map.get(&stock.symbol);
        if let Some(i) = option {
            self.stocks[*i] = stock;
            self.refresh_stock_index(*i)
        } else {
            let symbol = stock.symbol.clone();
            self.stocks.push(stock);
            self.stock_index_map.insert(symbol, self.stocks.len() - 1);
            self.refresh_stock_index(self.stocks.len() - 1);
        }
    }

    fn refresh_command(&mut self, command: &str) {
        self.move_cursor(self.page_size + 1, 0);
        print!("{}", command);
    }
}

impl MonoRender {
    fn build_render_map(&mut self, width: usize, height: usize) -> Result<(), String> {
        self.page_size = height - 2;
        if width < 24 {
            Err("Width is too small".to_owned())
        } else if width < 51 {
            self.renders.clear();
            self.renders.push((width / 23 * 7, MonoRender::render_name));
            self.renders
                .push((width - width / 23 * 7, MonoRender::render_last));
            Ok(())
        } else {
            self.renders.clear();
            self.renders.push((width / 51 * 7, MonoRender::render_name));
            self.renders.push((width / 51 * 8, MonoRender::render_open));
            self.renders
                .push((width / 51 * 16, MonoRender::render_last));
            self.renders.push((width / 51 * 8, MonoRender::render_low));
            self.renders.push((width / 51 * 8, MonoRender::render_high));
            Ok(())
        }
    }

    fn refresh_stock_index(&mut self, index: usize) {
        let cur_page_no = index / self.page_size;
        if cur_page_no + 1 != self.page_no {
            return;
        }

        if self.stocks.len() <= index {
            return;
        }

        let line = index - cur_page_no * self.page_size + 1;
        self.move_cursor(line, 0);

        for render in &self.renders {
            let (limit, func) = render;
            func(self.stocks.get(index), *limit);
        }
        self.stdout.flush().unwrap();
    }

    /// min limit is 7
    fn render_name(stock_option: Option<&Stock>, limit: usize) {
        if let Some(stock) = stock_option {
            print!("{:>width$}", stock.symbol, width = limit);
        } else {
            print!("{:>width$}", "symbol", width = limit)
        }
    }

    /// min limit is 8
    fn render_open(stock_option: Option<&Stock>, limit: usize) {
        if let Some(stock) = stock_option {
            print!("{:>width$}", stock.open, width = limit)
        } else {
            print!("{:>width$}", "open", width = limit)
        }
    }

    /// min limit is 16
    fn render_last(stock_option: Option<&Stock>, limit: usize) {
        if let Some(stock) = stock_option {
            let percent = (stock.last_done - stock.open) * Decimal::from(100) / stock.open;
            let last_done_show = format!("{:.3}({:.2}%)", stock.last_done, percent);
            print!("{:>width$}", last_done_show, width = limit)
        } else {
            print!("{:>width$}", "current", width = limit)
        }
    }

    /// min limit is 8
    fn render_low(stock_option: Option<&Stock>, limit: usize) {
        if let Some(stock) = stock_option {
            print!("{:>width$}", stock.low, width = limit)
        } else {
            print!("{:>width$}", "low", width = limit)
        }
    }

    /// min limit is 8
    fn render_high(stock_option: Option<&Stock>, limit: usize) {
        if let Some(stock) = stock_option {
            print!("{:>width$}", stock.high, width = limit)
        } else {
            print!("{:>width$}", "high", width = limit)
        }
    }

    fn move_cursor(&mut self, row: usize, col: usize) {
        execute!(self.stdout, cursor::MoveTo(col as u16, row as u16)).unwrap();
        execute!(self.stdout, Clear(ClearType::CurrentLine)).unwrap();
    }
}
