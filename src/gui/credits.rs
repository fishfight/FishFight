use crate::is_gamepad_btn_pressed;
use fishsticks::{Button, GamepadContext};
use macroquad::{experimental::collections::storage, prelude::*};
use std::path::Path;

const TEXT_X_OFFSET: f32 = 370.0;
const MAIN_HEADER_Y_OFFSET: f32 = 300.0;
const SECONDARY_HEADER_Y_OFFSET: f32 = 150.0;
const TEXT_Y_OFFSET: f32 = 60.0;

const CREDITS_LIST: [(&str, LabelType); 31] = [
    ("Fish Fight", LabelType::MainHeader),
    ("Founding Team", LabelType::SecondaryHeader),
    ("Erlend Sogge Heggen - General Director", LabelType::Regular),
    ("'Emerald Jak' - Sound Director", LabelType::Regular),
    (
        "Fedor Logachev - Technical Director (former)",
        LabelType::Regular,
    ),
    (
        "Ole A. Sjo Fasting - Technical Director (current)",
        LabelType::Regular,
    ),
    ("Orlando Herrera - Art Director", LabelType::Regular),
    ("Regulars", LabelType::SecondaryHeader),
    ("Orhun Parmaksız - Infrastructure", LabelType::Regular),
    ("Kultaev Aleksandr - Gameplay Code", LabelType::Regular),
    ("Carlo Supina - Dev Advocacy", LabelType::Regular),
    ("Micah Tigley - Dev Advocacy", LabelType::Regular),
    ("Drake - Media Content", LabelType::Regular),
    (
        "Michał Grzegórzek-Kiaszewicz - Gameplay Design",
        LabelType::Regular,
    ),
    ("Contributors", LabelType::SecondaryHeader),
    ("Saverio Miroddi - Gameplay Code", LabelType::Regular),
    ("@grufkork - Gameplay Code", LabelType::Regular),
    ("@PotatoTech - Gameplay Code", LabelType::Regular),
    ("Armando Gonzalez - Gameplay Code", LabelType::Regular),
    ("Isaac - Gameplay Code", LabelType::Regular),
    ("Kadir Yazıcı - Gameplay Code", LabelType::Regular),
    ("Ignat Insarovi - Gameplay Code", LabelType::Regular),
    ("garoslaw - Audio", LabelType::Regular),
    ("Animesh Sahu - Infrastructure", LabelType::Regular),
    ("Srayan Jana - Dev Advocacy", LabelType::Regular),
    ("Alve Larsson - Website", LabelType::Regular),
    ("William Batista - Gameplay Code", LabelType::Regular),
    ("Tech Foundations", LabelType::SecondaryHeader),
    ("macroquad.rs", LabelType::Regular),
    ("rust-lang.org", LabelType::Regular),
    ("Thank you!", LabelType::MainHeader),
];

#[derive(Clone)]
struct CreditLabel {
    text: String,
    x: f32,
    y: f32,
    font_size: u16,
}

#[derive(Clone, Copy)]
enum LabelType {
    MainHeader,
    SecondaryHeader,
    Regular,
}

pub async fn show_game_credits(assets_dir: &str) {
    let gamepad_context = storage::get::<GamepadContext>();

    let mut delta = 200.0;
    let credits = create_game_credits();

    let font = load_ttf_font(
        Path::new(assets_dir)
            .join("ui/AnonymousPro-Regular.ttf")
            .to_str()
            .unwrap(),
    )
    .await
    .unwrap();

    loop {
        if is_key_pressed(KeyCode::Escape)
            || is_gamepad_btn_pressed(Some(&gamepad_context), Button::East)
        {
            break;
        }

        clear_background(BLACK);
        delta -= 0.5;

        for credit in &credits {
            let x = screen_width() / 2.0 - credit.x;
            let y = credit.y + delta;
            draw_text_ex(
                &credit.text,
                x,
                y,
                TextParams {
                    font,
                    font_size: credit.font_size,
                    color: WHITE,
                    ..Default::default()
                },
            );
        }

        next_frame().await;
    }
}

fn create_game_credits() -> Vec<CreditLabel> {
    let mut game_credits: Vec<CreditLabel> = Vec::new();
    let mut prev_position = 0.0;

    for credit in CREDITS_LIST {
        let credit_label = credit;
        match credit_label.1 {
            LabelType::MainHeader => {
                // The "Fish Fight" header sets the initial position
                if credit.0 == "Fish Fight" {
                    prev_position -= MAIN_HEADER_Y_OFFSET;
                } else {
                    prev_position -= -MAIN_HEADER_Y_OFFSET;
                }

                game_credits.push(CreditLabel {
                    text: credit_label.0.to_string(),
                    x: TEXT_X_OFFSET,
                    y: screen_height() + prev_position,
                    font_size: 100,
                });
            }
            LabelType::SecondaryHeader => {
                prev_position -= -SECONDARY_HEADER_Y_OFFSET;

                game_credits.push(CreditLabel {
                    text: credit_label.0.to_string(),
                    x: TEXT_X_OFFSET,
                    y: screen_height() + prev_position,
                    font_size: 40,
                });
            }
            LabelType::Regular => {
                prev_position -= -TEXT_Y_OFFSET;

                game_credits.push(CreditLabel {
                    text: credit_label.0.to_string(),
                    x: TEXT_X_OFFSET,
                    y: screen_height() + prev_position,
                    font_size: 30,
                });
            }
        }
    }

    game_credits
}
