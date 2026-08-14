use gtk::gdk::{self, ModifierType};
use gtk::glib::Propagation;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, ComboBoxText, Entry, Grid, Label,
    Orientation,
};
use rust_calc::{
    HistoryStore, I18n, Language, Operator, calculate, format_number, resolve_database_path,
};
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
    history_label.set_yalign(0.0);
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
        .row_spacing(8)
        .column_spacing(8)
        .column_homogeneous(true)
        .row_homogeneous(true)
        .build();
    grid.style_context().add_class("keypad");

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
        button.style_context().add_class("digit");
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
        .default_width(390)
        .default_height(700)
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

    let lang_combo = ComboBoxText::new();
    lang_combo.style_context().add_class("lang-combo");
    lang_combo.set_tooltip_text(Some("Language / Dil / Язык"));
    lang_combo.set_size_request(128, -1);
    for lang in Language::all() {
        lang_combo.append(Some(lang.code()), lang.display_name());
    }
    lang_combo.set_active_id(Some(initial_lang.code()));

    let history_eyebrow = Label::new(Some(I18n::recent_operations(initial_lang)));
    history_eyebrow.set_xalign(0.0);
    history_eyebrow.style_context().add_class("eyebrow");

    let history_lang_store = history.clone();
    let history_lang_label = history_label.clone();
    let history_lang_eyebrow = history_eyebrow.clone();
    let history_lang_btn = history_btn.clone();
    let window_lang = window.clone();

    lang_combo.connect_changed(move |combo| {
        if let Some(id) = combo.active_id() {
            let new_lang = Language::from_code(&id);
            if let Err(e) = history_lang_store.borrow_mut().set_language(new_lang) {
                eprintln!("Dil kaydedilemedi: {e}");
            }
            window_lang.set_title(I18n::app_title(new_lang));
            history_lang_eyebrow.set_text(I18n::recent_operations(new_lang));
            history_lang_btn.set_label(I18n::history_btn_label(new_lang));
            refresh_history(
                &history_lang_label,
                &history_lang_store.borrow().entry_texts(),
                new_lang,
            );
        }
    });

    let brand_mark = Label::new(Some("R"));
    brand_mark.style_context().add_class("brand-mark");

    let brand_copy = GtkBox::new(Orientation::Vertical, 0);
    let brand_title = Label::new(Some("RUSTCALC"));
    brand_title.set_xalign(0.0);
    brand_title.style_context().add_class("brand-title");
    let brand_subtitle = Label::new(Some("FERRITEDB  •  LOCAL WAL"));
    brand_subtitle.set_xalign(0.0);
    brand_subtitle.style_context().add_class("brand-subtitle");
    brand_copy.pack_start(&brand_title, false, false, 0);
    brand_copy.pack_start(&brand_subtitle, false, false, 0);

    let brand_bar = GtkBox::new(Orientation::Horizontal, 10);
    brand_bar.style_context().add_class("brand-bar");
    brand_bar.pack_start(&brand_mark, false, false, 0);
    brand_bar.pack_start(&brand_copy, true, true, 0);
    brand_bar.pack_start(&lang_combo, false, false, 0);

    let history_box = GtkBox::new(Orientation::Vertical, 4);
    history_box.pack_start(&history_eyebrow, false, false, 0);
    history_box.pack_start(&history_label, true, true, 0);

    let display_box = GtkBox::new(Orientation::Vertical, 4);
    display_box.style_context().add_class("display-panel");
    display_box.pack_start(&history_box, true, true, 0);
    display_box.pack_start(&formula_label, false, false, 0);
    display_box.pack_start(&display, false, false, 0);

    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_border_width(16);
    content.style_context().add_class("app-shell");
    content.pack_start(&brand_bar, false, false, 0);
    content.pack_start(&display_box, true, true, 0);
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
    if let Some(close_button) = dialog.widget_for_response(gtk::ResponseType::Close) {
        close_button.style_context().add_class("dialog-close");
    }

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
    resolve_database_path(
        std::env::var_os("RUST_CALC_DATA").map(PathBuf::from),
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

fn install_css() {
    let css = gtk::CssProvider::new();
    css.load_from_data(
        b"window { background-color: #080c0f; color: #f5e8d8; }\n\
          .app-shell { background-color: #080c0f; }\n\
          .brand-bar { min-height: 38px; }\n\
          .brand-mark { min-width: 34px; min-height: 34px; border-radius: 18px; border: 1px solid #d59a64; background-image: linear-gradient(to bottom, #493326, #201a18); color: #ffd8a8; font-size: 18px; font-weight: 800; }\n\
          .brand-title { color: #f0c89d; font-size: 15px; font-weight: 800; }\n\
          .brand-subtitle { color: #609a99; font-size: 9px; font-weight: 700; }\n\
          .display-panel { min-height: 142px; padding: 12px 14px; border-radius: 14px; border: 1px solid #2c7775; background-image: linear-gradient(145deg, #112326, #0b1418 70%); box-shadow: inset 0 1px rgba(117, 220, 211, 0.12), 0 3px 8px rgba(0, 0, 0, 0.35); }\n\
          .eyebrow { color: #5b9f9d; font-size: 9px; font-weight: 800; }\n\
          label#history { color: #b9c8c3; font-family: monospace; font-size: 11px; min-height: 48px; }\n\
          label#formula { color: #c08d62; font-size: 13px; font-weight: 600; min-height: 18px; margin-right: 3px; }\n\
          combobox.lang-combo button { min-height: 34px; padding: 0 10px; border-radius: 8px; border: 1px solid #29494a; background-image: linear-gradient(to bottom, #1a292d, #10181c); color: #d8d7cd; font-size: 11px; font-weight: 600; }\n\
          combobox.lang-combo button:hover { border-color: #4a8c89; background-image: linear-gradient(to bottom, #20373a, #142125); }\n\
          entry#display { font-size: 38px; font-weight: 700; color: #ffe0b8; caret-color: transparent; background: transparent; border: none; box-shadow: none; padding: 2px 0 0 0; text-shadow: 0 1px #5c3525; }\n\
          .keypad { margin-top: 1px; }\n\
          button { font-size: 20px; font-weight: 600; min-height: 52px; border-radius: 10px; border: 1px solid #3a4c4f; background-image: linear-gradient(to bottom, #2a3437, #151b1f 72%); color: #f4d8b6; box-shadow: inset 0 1px rgba(255, 255, 255, 0.12), 0 2px 3px rgba(0, 0, 0, 0.55); text-shadow: 0 1px #3e261c; }\n\
          button:hover { border-color: #6a9b96; background-image: linear-gradient(to bottom, #354448, #1c2529 72%); }\n\
          button:active { background-image: linear-gradient(to bottom, #12191c, #273236); box-shadow: inset 0 2px 3px rgba(0, 0, 0, 0.5); }\n\
          button:focus { border-color: #76d5ce; box-shadow: 0 0 0 1px rgba(118, 213, 206, 0.35); }\n\
          button.operator { border-color: #356c6c; color: #72d7d2; font-weight: 700; }\n\
          button.operator:hover { border-color: #62bcb7; color: #b9fffa; }\n\
          button.clear-btn { border-color: #7a3341; background-image: linear-gradient(to bottom, #521d2a, #260f18); color: #ff8798; font-weight: 700; text-shadow: none; }\n\
          button.clear-btn:hover { border-color: #b6495d; background-image: linear-gradient(to bottom, #6b2636, #35131f); }\n\
          button.equals { min-height: 58px; border-color: #eeaa70; background-image: linear-gradient(to right, #6f3e29, #b66d43 50%, #6f3e29); color: #fff4e7; font-weight: 800; box-shadow: inset 0 1px rgba(255, 232, 201, 0.5), 0 2px 5px rgba(0, 0, 0, 0.55); }\n\
          button.equals:hover { border-color: #ffd0a0; background-image: linear-gradient(to right, #855038, #ce8253 50%, #855038); }\n\
          button.history-btn { font-size: 12px; font-weight: 600; min-height: 40px; border-radius: 9px; border-color: #29494a; background-image: linear-gradient(to bottom, #18272b, #0e171b); color: #b8cfca; text-shadow: none; }\n\
          button.history-btn:hover { border-color: #4d8f8b; color: #e4f4ef; }\n\
          dialog, dialog box { background-color: #0b1115; color: #eadfce; }\n\
          button.dialog-close { min-height: 36px; padding: 0 18px; border-radius: 8px; border-color: #3c5557; font-size: 13px; font-weight: 600; text-shadow: none; }\n\
          .stats-card { background-image: linear-gradient(145deg, #142427, #0d171a); border: 1px solid #356c6c; border-radius: 10px; padding: 12px; color: #c7d6d1; }\n\
          .history-list { background-color: #0a1013; border: 1px solid #24393b; border-radius: 8px; }\n\
          .history-list row:hover { background-color: #172629; }\n\
          .history-list row:selected { background-color: #234345; color: #fff0dc; }\n\
          .history-item-text { font-family: monospace; font-size: 14px; color: #eed5b8; }\n\
          .empty-label { color: #718b89; font-size: 13px; }\n\
          button.destructive-action { background-image: linear-gradient(to bottom, #5b202d, #35131c); border-color: #8b394a; color: #ffacb8; font-size: 13px; font-weight: 700; min-height: 40px; border-radius: 8px; text-shadow: none; }\n\
          button.destructive-action:hover { border-color: #c55268; background-image: linear-gradient(to bottom, #732a3a, #431923); }",
    )
    .expect("CSS yüklenemedi");
    gtk::StyleContext::add_provider_for_screen(
        &gtk::gdk::Screen::default().expect("ekran bulunamadı"),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
