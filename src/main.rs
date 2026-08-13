use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, Grid, Label, Orientation};
use rust_calc::{HistoryStore, Operator, calculate, format_number};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const APP_ID: &str = "dev.koray.rustcalc";

#[derive(Default)]
struct CalculatorState {
    left: Option<f64>,
    operator: Option<Operator>,
    replace_display: bool,
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let history = match HistoryStore::open(database_path()) {
        Ok(history) => Rc::new(RefCell::new(history)),
        Err(error) => {
            eprintln!("FerriteDB açılamadı: {error}");
            app.quit();
            return;
        }
    };

    let state = Rc::new(RefCell::new(CalculatorState::default()));
    let display = Entry::builder()
        .text("0")
        .editable(false)
        .xalign(1.0)
        .build();
    display.set_widget_name("display");

    let history_label = Label::new(None);
    history_label.set_xalign(0.0);
    history_label.set_widget_name("history");
    refresh_history(&history_label, &history.borrow().entries());

    let grid = Grid::builder()
        .row_spacing(8)
        .column_spacing(8)
        .column_homogeneous(true)
        .row_homogeneous(true)
        .build();

    for (label, col, row) in [
        ("7", 0, 0),
        ("8", 1, 0),
        ("9", 2, 0),
        ("4", 0, 1),
        ("5", 1, 1),
        ("6", 2, 1),
        ("1", 0, 2),
        ("2", 1, 2),
        ("3", 2, 2),
        ("0", 0, 3),
        (",", 1, 3),
    ] {
        let button = Button::with_label(label);
        let display = display.clone();
        let state = state.clone();
        button.connect_clicked(move |_| append_digit(&display, &state, label));
        grid.attach(&button, col, row, 1, 1);
    }

    for (label, operator, row) in [
        ("÷", Operator::Divide, 0),
        ("×", Operator::Multiply, 1),
        ("−", Operator::Subtract, 2),
        ("+", Operator::Add, 3),
    ] {
        let button = Button::with_label(label);
        button.style_context().add_class("operator");
        let display = display.clone();
        let state = state.clone();
        button.connect_clicked(move |_| select_operator(&display, &state, operator));
        grid.attach(&button, 3, row, 1, 1);
    }

    let clear = Button::with_label("C");
    let clear_display = display.clone();
    let clear_state = state.clone();
    clear.connect_clicked(move |_| {
        clear_display.set_text("0");
        *clear_state.borrow_mut() = CalculatorState::default();
    });
    grid.attach(&clear, 2, 3, 1, 1);

    let equals = Button::with_label("=");
    equals.style_context().add_class("equals");
    let equals_display = display.clone();
    let equals_state = state.clone();
    let equals_history = history.clone();
    let equals_history_label = history_label.clone();
    equals.connect_clicked(move |_| {
        evaluate(
            &equals_display,
            &equals_state,
            &equals_history,
            &equals_history_label,
        );
    });
    grid.attach(&equals, 0, 4, 4, 1);

    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_border_width(18);
    content.pack_start(&history_label, false, false, 0);
    content.pack_start(&display, false, false, 0);
    content.pack_start(&grid, true, true, 0);

    install_css();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ferrite Hesap Makinesi")
        .default_width(340)
        .default_height(490)
        .resizable(false)
        .child(&content)
        .build();
    window.show_all();
}

fn append_digit(display: &Entry, state: &Rc<RefCell<CalculatorState>>, input: &str) {
    let mut state = state.borrow_mut();
    let mut text = if state.replace_display || display.text() == "0" {
        String::new()
    } else {
        display.text().to_string()
    };
    state.replace_display = false;

    if input == "," {
        if text.contains(',') {
            return;
        }
        if text.is_empty() {
            text.push('0');
        }
    }
    text.push_str(input);
    display.set_text(&text);
}

fn select_operator(display: &Entry, state: &Rc<RefCell<CalculatorState>>, operator: Operator) {
    if let Ok(value) = parse_display(display) {
        let mut state = state.borrow_mut();
        state.left = Some(value);
        state.operator = Some(operator);
        state.replace_display = true;
    }
}

fn evaluate(
    display: &Entry,
    state: &Rc<RefCell<CalculatorState>>,
    history: &Rc<RefCell<HistoryStore>>,
    history_label: &Label,
) {
    let right = match parse_display(display) {
        Ok(value) => value,
        Err(message) => {
            display.set_text(message);
            return;
        }
    };
    let (left, operator) = {
        let state = state.borrow();
        match (state.left, state.operator) {
            (Some(left), Some(operator)) => (left, operator),
            _ => return,
        }
    };

    match calculate(left, operator, right) {
        Ok(result) if result.is_finite() => {
            display.set_text(&format_number(result));
            if let Err(error) = history.borrow_mut().record(left, operator, right, result) {
                eprintln!("Geçmiş kaydedilemedi: {error}");
            }
            refresh_history(history_label, &history.borrow().entries());
            *state.borrow_mut() = CalculatorState {
                left: Some(result),
                operator: None,
                replace_display: true,
            };
        }
        Ok(_) => display.set_text("Sonuç çok büyük"),
        Err(message) => display.set_text(message),
    }
}

fn parse_display(display: &Entry) -> Result<f64, &'static str> {
    display
        .text()
        .replace(',', ".")
        .parse()
        .map_err(|_| "Geçersiz sayı")
}

fn refresh_history(label: &Label, entries: &[String]) {
    let recent = entries
        .iter()
        .rev()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    label.set_text(if recent.is_empty() {
        "FerriteDB geçmişi boş"
    } else {
        &recent
    });
}

fn database_path() -> PathBuf {
    std::env::var_os("RUST_CALC_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/history.ferrite"))
}

fn install_css() {
    let css = gtk::CssProvider::new();
    css.load_from_data(
        b"window { background: #17191f; }\n\
          entry#display { font-size: 34px; padding: 16px; }\n\
          label#history { color: #9da3b4; min-height: 58px; }\n\
          button { font-size: 20px; min-height: 54px; border-radius: 10px; }\n\
          button.operator { color: #7db7ff; }\n\
          button.equals { background: #3979d5; color: white; }",
    )
    .expect("CSS yüklenemedi");
    gtk::StyleContext::add_provider_for_screen(
        &gtk::gdk::Screen::default().expect("ekran bulunamadı"),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
