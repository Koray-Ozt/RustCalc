use gtk::gdk::{self, ModifierType};
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, ComboBoxText, Entry, Grid, Label,
    Orientation,
};
use rust_calc::{HistoryStore, I18n, Language, Operator, calculate, format_number};
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

    let initial_lang = history.borrow().language();
    let state = Rc::new(RefCell::new(CalculatorState::default()));

    let history_label = Label::new(None);
    history_label.set_xalign(0.0);
    history_label.set_widget_name("history");
    refresh_history(
        &history_label,
        &history.borrow().entry_texts(),
        initial_lang,
    );

    let formula_label = Label::new(Some(""));
    formula_label.set_xalign(1.0);
    formula_label.set_widget_name("formula");

    let display = Entry::builder()
        .text("0")
        .editable(false)
        .xalign(1.0)
        .build();
    display.set_widget_name("display");

    let grid = Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
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
        let formula_label = formula_label.clone();
        let state = state.clone();
        let history_op = history.clone();
        button.connect_clicked(move |_| {
            let lang = history_op.borrow().language();
            select_operator(&display, &formula_label, &state, operator, lang)
        });
        grid.attach(&button, 3, row, 1, 1);
    }

    let clear = Button::with_label("C");
    clear.style_context().add_class("clear-btn");
    let clear_display = display.clone();
    let clear_formula = formula_label.clone();
    let clear_state = state.clone();
    clear.connect_clicked(move |_| {
        clear_display.set_text("0");
        clear_formula.set_text("");
        *clear_state.borrow_mut() = CalculatorState::default();
    });
    grid.attach(&clear, 2, 3, 1, 1);

    let equals = Button::with_label("=");
    equals.style_context().add_class("equals");
    let equals_display = display.clone();
    let equals_formula = formula_label.clone();
    let equals_state = state.clone();
    let equals_history = history.clone();
    let equals_history_label = history_label.clone();
    equals.connect_clicked(move |_| {
        evaluate(
            &equals_display,
            &equals_formula,
            &equals_state,
            &equals_history,
            &equals_history_label,
        );
    });
    grid.attach(&equals, 0, 4, 4, 1);

    install_css();
    let window = ApplicationWindow::builder()
        .application(app)
        .title(I18n::app_title(initial_lang))
        .default_width(360)
        .default_height(620)
        .resizable(false)
        .build();

    let history_btn = Button::with_label(I18n::history_btn_label(initial_lang));
    history_btn.style_context().add_class("history-btn");

    let history_btn_window = window.clone();
    let history_btn_store = history.clone();
    let history_btn_label = history_label.clone();
    let history_btn_display = display.clone();

    history_btn.connect_clicked(move |_| {
        show_history_dialog(
            &history_btn_window,
            &history_btn_store,
            &history_btn_label,
            &history_btn_display,
        );
    });

    // Language Selector Header Bar
    let lang_box = GtkBox::new(Orientation::Horizontal, 8);
    lang_box.set_halign(gtk::Align::End);

    let lang_combo = ComboBoxText::new();
    lang_combo.style_context().add_class("lang-combo");
    for lang in Language::all() {
        lang_combo.append(Some(lang.code()), lang.display_name());
    }
    lang_combo.set_active_id(Some(initial_lang.code()));

    let history_lang_store = history.clone();
    let history_lang_label = history_label.clone();
    let history_lang_btn = history_btn.clone();
    let window_lang = window.clone();

    lang_combo.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            let new_lang = Language::from_code(&id);
            if let Err(e) = history_lang_store.borrow_mut().set_language(new_lang) {
                eprintln!("Dil kaydedilemedi: {e}");
            }
            window_lang.set_title(I18n::app_title(new_lang));
            history_lang_btn.set_label(I18n::history_btn_label(new_lang));
            refresh_history(
                &history_lang_label,
                &history_lang_store.borrow().entry_texts(),
                new_lang,
            );
        }
    });

    lang_box.pack_start(&lang_combo, false, false, 0);

    let top_bar = GtkBox::new(Orientation::Horizontal, 8);
    top_bar.pack_start(&history_label, true, true, 0);
    top_bar.pack_start(&lang_box, false, false, 0);

    let display_box = GtkBox::new(Orientation::Vertical, 4);
    display_box.pack_start(&formula_label, false, false, 0);
    display_box.pack_start(&display, false, false, 0);

    let content = GtkBox::new(Orientation::Vertical, 14);
    content.set_border_width(18);
    content.pack_start(&top_bar, false, false, 0);
    content.pack_start(&display_box, false, false, 0);
    content.pack_start(&grid, true, true, 0);
    content.pack_start(&history_btn, false, false, 0);

    window.add(&content);

    // Keyboard Shortcuts Listener
    let key_display = display.clone();
    let key_formula = formula_label.clone();
    let key_state = state.clone();
    let key_history = history.clone();
    let key_history_label = history_label.clone();
    let key_window = window.clone();

    window.connect_key_press_event(move |_, event| {
        let keyval = event.keyval();
        let state_flags = event.state();
        let ch = keyval.to_unicode();

        if state_flags.contains(ModifierType::CONTROL_MASK) && (ch == Some('h') || ch == Some('H'))
        {
            show_history_dialog(&key_window, &key_history, &key_history_label, &key_display);
            return Propagation::Stop;
        }

        match keyval {
            gdk::keys::constants::Return | gdk::keys::constants::KP_Enter => {
                evaluate(
                    &key_display,
                    &key_formula,
                    &key_state,
                    &key_history,
                    &key_history_label,
                );
                Propagation::Stop
            }
            gdk::keys::constants::BackSpace => {
                handle_backspace(&key_display, &key_state);
                Propagation::Stop
            }
            gdk::keys::constants::Escape => {
                key_display.set_text("0");
                key_formula.set_text("");
                *key_state.borrow_mut() = CalculatorState::default();
                Propagation::Stop
            }
            _ => {
                if let Some(c) = ch {
                    match c {
                        '0'..='9' => {
                            let digit = c.to_string();
                            append_digit(&key_display, &key_state, &digit);
                            Propagation::Stop
                        }
                        ',' | '.' => {
                            append_digit(&key_display, &key_state, ",");
                            Propagation::Stop
                        }
                        '+' => {
                            let lang = key_history.borrow().language();
                            select_operator(
                                &key_display,
                                &key_formula,
                                &key_state,
                                Operator::Add,
                                lang,
                            );
                            Propagation::Stop
                        }
                        '-' => {
                            let lang = key_history.borrow().language();
                            select_operator(
                                &key_display,
                                &key_formula,
                                &key_state,
                                Operator::Subtract,
                                lang,
                            );
                            Propagation::Stop
                        }
                        '*' => {
                            let lang = key_history.borrow().language();
                            select_operator(
                                &key_display,
                                &key_formula,
                                &key_state,
                                Operator::Multiply,
                                lang,
                            );
                            Propagation::Stop
                        }
                        '/' => {
                            let lang = key_history.borrow().language();
                            select_operator(
                                &key_display,
                                &key_formula,
                                &key_state,
                                Operator::Divide,
                                lang,
                            );
                            Propagation::Stop
                        }

                        '=' => {
                            evaluate(
                                &key_display,
                                &key_formula,
                                &key_state,
                                &key_history,
                                &key_history_label,
                            );
                            Propagation::Stop
                        }
                        'c' | 'C' => {
                            key_display.set_text("0");
                            key_formula.set_text("");
                            *key_state.borrow_mut() = CalculatorState::default();
                            Propagation::Stop
                        }
                        'h' | 'H' => {
                            show_history_dialog(
                                &key_window,
                                &key_history,
                                &key_history_label,
                                &key_display,
                            );
                            Propagation::Stop
                        }
                        _ => Propagation::Proceed,
                    }
                } else {
                    Propagation::Proceed
                }
            }
        }
    });

    window.show_all();
}

fn handle_backspace(display: &Entry, state: &Rc<RefCell<CalculatorState>>) {
    let state = state.borrow();
    if state.replace_display {
        return;
    }
    let text = display.text().to_string();
    if text.len() <= 1 || (text.len() == 2 && text.starts_with('-')) {
        display.set_text("0");
    } else {
        let mut new_text = text;
        new_text.pop();
        display.set_text(&new_text);
    }
}

fn show_history_dialog(
    parent: &ApplicationWindow,
    history: &Rc<RefCell<HistoryStore>>,
    history_label: &Label,
    main_display: &Entry,
) {
    let lang = history.borrow().language();
    let dialog = gtk::Dialog::with_buttons(
        Some(I18n::history_dialog_title(lang)),
        Some(parent),
        gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
        &[(I18n::close(lang), gtk::ResponseType::Close)],
    );
    dialog.set_default_size(380, 460);

    let content_area = dialog.content_area();
    content_area.set_spacing(14);
    content_area.set_border_width(16);

    let stats = history.borrow().stats();
    let most_used = stats
        .most_used_operator()
        .map(|op| op.symbol())
        .unwrap_or_else(|| I18n::none(lang));

    let stats_card = GtkBox::new(Orientation::Vertical, 6);
    stats_card.style_context().add_class("stats-card");

    let stats_title = Label::new(None);
    stats_title.set_markup(&format!("<b>{}</b>", I18n::analytics_title(lang)));
    stats_title.set_xalign(0.0);
    stats_card.pack_start(&stats_title, false, false, 0);

    let stats_text = format!(
        "• <b>{}:</b> {}\n\
         • <b>{}:</b> {}\n\
         • <b>{}:</b> + ({})  |  − ({})  |  × ({})  |  ÷ ({})",
        I18n::total_count(lang),
        stats.total_count,
        I18n::favorite_operator(lang),
        most_used,
        I18n::breakdown(lang),
        stats.add_count,
        stats.subtract_count,
        stats.multiply_count,
        stats.divide_count
    );

    let stats_body = Label::new(None);
    stats_body.set_markup(&stats_text);
    stats_body.set_xalign(0.0);
    stats_card.pack_start(&stats_body, false, false, 0);

    content_area.pack_start(&stats_card, false, false, 0);

    let list_title = Label::new(None);
    list_title.set_markup(&format!(
        "<small><b>{}</b></small>",
        I18n::history_list_title(lang)
    ));
    list_title.set_xalign(0.0);
    content_area.pack_start(&list_title, false, false, 0);

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .min_content_height(190)
        .build();

    let list_box = gtk::ListBox::new();
    list_box.style_context().add_class("history-list");

    let entries = history.borrow().entries().to_vec();
    if entries.is_empty() {
        let empty_label = Label::new(Some(I18n::no_history_records(lang)));
        empty_label.set_margin_top(16);
        empty_label.set_margin_bottom(16);
        empty_label.style_context().add_class("empty-label");
        list_box.add(&empty_label);
    } else {
        for entry in entries.iter().rev() {
            let row_box = GtkBox::new(Orientation::Horizontal, 8);
            row_box.set_margin_top(6);
            row_box.set_margin_bottom(6);
            row_box.set_margin_start(10);
            row_box.set_margin_end(10);

            let expr_label = Label::new(Some(&entry.formatted()));
            expr_label.set_xalign(0.0);
            expr_label.style_context().add_class("history-item-text");

            row_box.pack_start(&expr_label, true, true, 0);
            list_box.add(&row_box);
        }
    }

    let entries_clone = entries.clone();
    let main_display_clone = main_display.clone();
    let dialog_row_click = dialog.clone();

    list_box.connect_row_activated(move |_, row| {
        let index = row.index() as usize;
        if index < entries_clone.len() {
            let rev_index = entries_clone.len() - 1 - index;
            let target_entry = &entries_clone[rev_index];
            main_display_clone.set_text(&format_number(target_entry.result));
            dialog_row_click.response(gtk::ResponseType::Close);
        }
    });

    scrolled_window.add(&list_box);
    content_area.pack_start(&scrolled_window, true, true, 0);

    let clear_button = Button::with_label(I18n::clear_history(lang));
    clear_button.style_context().add_class("destructive-action");

    let history_clone = history.clone();
    let history_label_clone = history_label.clone();
    let dialog_clone = dialog.clone();

    clear_button.connect_clicked(move |_| {
        if let Err(err) = history_clone.borrow_mut().clear() {
            eprintln!("Geçmiş temizlenemedi: {err}");
        } else {
            refresh_history(
                &history_label_clone,
                &history_clone.borrow().entry_texts(),
                lang,
            );
            dialog_clone.response(gtk::ResponseType::Close);
        }
    });

    content_area.pack_start(&clear_button, false, false, 0);

    dialog.show_all();
    dialog.connect_response(|d, _| d.close());
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

fn select_operator(
    display: &Entry,
    formula_label: &Label,
    state: &Rc<RefCell<CalculatorState>>,
    operator: Operator,
    lang: Language,
) {
    if let Ok(value) = parse_display(display, lang) {
        let mut state = state.borrow_mut();
        state.left = Some(value);
        state.operator = Some(operator);
        state.replace_display = true;
        formula_label.set_text(&format!("{} {}", format_number(value), operator.symbol()));
    }
}

fn evaluate(
    display: &Entry,
    formula_label: &Label,
    state: &Rc<RefCell<CalculatorState>>,
    history: &Rc<RefCell<HistoryStore>>,
    history_label: &Label,
) {
    let lang = history.borrow().language();
    let right = match parse_display(display, lang) {
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

    formula_label.set_text(&format!(
        "{} {} {} =",
        format_number(left),
        operator.symbol(),
        format_number(right)
    ));

    match calculate(left, operator, right, lang) {
        Ok(result) if result.is_finite() => {
            display.set_text(&format_number(result));
            if let Err(error) = history.borrow_mut().record(left, operator, right, result) {
                eprintln!("Geçmiş kaydedilemedi: {error}");
            }
            refresh_history(history_label, &history.borrow().entry_texts(), lang);
            *state.borrow_mut() = CalculatorState {
                left: Some(result),
                operator: None,
                replace_display: true,
            };
        }
        Ok(_) => display.set_text(I18n::result_too_large(lang)),
        Err(message) => display.set_text(message),
    }
}

fn parse_display(display: &Entry, lang: Language) -> Result<f64, &'static str> {
    display
        .text()
        .replace(',', ".")
        .parse()
        .map_err(|_| I18n::invalid_number(lang))
}

fn refresh_history(label: &Label, entries: &[String], lang: Language) {
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
        I18n::history_empty(lang)
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
        b"window { background-color: #12151c; color: #f8fafc; }\n\
          label#history { color: #64748b; font-size: 12px; min-height: 52px; }\n\
          label#formula { color: #94a3b8; font-size: 15px; font-weight: 500; min-height: 22px; margin-right: 4px; }\n\
          combobox.lang-combo { background-color: #1e293b; color: #cbd5e1; border-radius: 6px; font-size: 12px; }\n\
          entry#display { font-size: 38px; font-weight: 600; color: #f8fafc; background-color: #0f1218; border: 1px solid #1e293b; border-radius: 12px; padding: 12px 16px; }\n\
          button { font-size: 20px; font-weight: 500; min-height: 54px; border-radius: 12px; background-color: #242b38; color: #f1f5f9; border: none; }\n\
          button:hover { background-color: #2d3646; }\n\
          button:active { background-color: #384356; }\n\
          button.operator { background-color: #1e293b; color: #38bdf8; font-weight: 600; }\n\
          button.operator:hover { background-color: #2563eb; color: #ffffff; }\n\
          button.clear-btn { background-color: #2e1d24; color: #f87171; font-weight: 600; }\n\
          button.clear-btn:hover { background-color: #451a23; }\n\
          button.equals { background-color: #2563eb; color: #ffffff; font-weight: 700; }\n\
          button.equals:hover { background-color: #1d4ed8; }\n\
          button.history-btn { font-size: 13px; font-weight: 500; min-height: 40px; background-color: #1e293b; color: #cbd5e1; border-radius: 8px; }\n\
          button.history-btn:hover { background-color: #334155; }\n\
          .stats-card { background-color: #1a1e28; border: 1px solid #334155; border-radius: 10px; padding: 12px; color: #cbd5e1; }\n\
          .history-list { background-color: #12151c; border-radius: 8px; }\n\
          .history-item-text { font-family: monospace; font-size: 14px; color: #e2e8f0; }\n\
          .empty-label { color: #64748b; font-size: 13px; }\n\
          button.destructive-action { background-color: #dc2626; color: white; font-size: 14px; font-weight: 600; min-height: 42px; border-radius: 8px; }\n\
          button.destructive-action:hover { background-color: #b91c1c; }",
    )
    .expect("CSS yüklenemedi");
    gtk::StyleContext::add_provider_for_screen(
        &gtk::gdk::Screen::default().expect("ekran bulunamadı"),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
