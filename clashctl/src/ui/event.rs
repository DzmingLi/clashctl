use std::fmt::Display;

use clashctl_core::model::{ConnectionsWithSpeed, Log, Proxies, Rules, Traffic, Version};
use crossterm::event::{KeyCode as KC, KeyEvent as KE, KeyModifiers as KM};
use log::Level;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use crate::{
    ui::{
        components::MovableListItem,
        config::get_config,
        keybind::{match_tab_goto, matches_any},
        utils::AsColor,
        TuiError, TuiResult,
    },
    Action,
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Event {
    Quit,
    Action(Action),
    Input(InputEvent),
    Update(UpdateEvent),
    Diagnostic(DiagnosticEvent),
}

impl<'a> MovableListItem<'a> for Event {
    fn to_spans(&self) -> Spans<'a> {
        match self {
            Event::Quit => Spans(vec![]),
            Event::Action(action) => Spans(vec![
                Span::styled("⋉ ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:?}", action)),
            ]),
            Event::Update(event) => Spans(vec![
                Span::styled("⇵  ", Style::default().fg(Color::Yellow)),
                Span::raw(event.to_string()),
            ]),
            Event::Input(event) => Spans(vec![
                Span::styled("✜  ", Style::default().fg(Color::Green)),
                Span::raw(format!("{:?}", event)),
            ]),
            Event::Diagnostic(event) => match event {
                DiagnosticEvent::Log(level, payload) => Spans(vec![
                    Span::styled(
                        format!("✇  {:<6}", level),
                        Style::default().fg(level.as_color()),
                    ),
                    Span::raw(payload.to_owned()),
                ]),
            },
        }
    }
}

impl Event {
    pub fn is_quit(&self) -> bool {
        matches!(self, Event::Quit)
    }

    pub fn is_interface(&self) -> bool {
        matches!(self, Event::Input(_))
    }

    pub fn is_update(&self) -> bool {
        matches!(self, Event::Update(_))
    }

    pub fn is_diagnostic(&self) -> bool {
        matches!(self, Event::Diagnostic(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputEvent {
    Esc,
    TabGoto(u8),
    ToggleDebug,
    ToggleHold,
    List(ListEvent),
    TestLatency,
    NextSort,
    PrevSort,
    RefreshSubscription,
    Other(KE),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListEvent {
    pub fast: bool,
    pub code: KC,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UpdateEvent {
    Config(crate::interactive::clashctl::model::Config),
    Connection(ConnectionsWithSpeed),
    Version(Version),
    Traffic(Traffic),
    Proxies(Proxies),
    Rules(Rules),
    Log(Log),
    ProxyTestLatencyDone,
    SubscriptionRefreshResult(String),
}

impl Display for UpdateEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateEvent::Config(x) => write!(f, "{:?}", x),
            UpdateEvent::Connection(x) => write!(f, "{:?}", x),
            UpdateEvent::Version(x) => write!(f, "{:?}", x),
            UpdateEvent::Traffic(x) => write!(f, "{:?}", x),
            UpdateEvent::Proxies(x) => write!(f, "{:?}", x),
            UpdateEvent::Rules(x) => write!(f, "{:?}", x),
            UpdateEvent::Log(x) => write!(f, "{:?}", x),
            UpdateEvent::ProxyTestLatencyDone => write!(f, "Test latency done"),
            UpdateEvent::SubscriptionRefreshResult(ref msg) => write!(f, "Subscription: {}", msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticEvent {
    Log(Level, String),
}

impl From<KE> for Event {
    fn from(value: KE) -> Self {
        let bindings = &get_config().keybindings;

        // Configurable bindings (checked first)
        if matches_any(&value, &bindings.quit) {
            return Self::Quit;
        }
        if matches_any(&value, &bindings.toggle_debug) {
            return Self::Input(InputEvent::ToggleDebug);
        }
        if matches_any(&value, &bindings.refresh_subscription) {
            return Self::Input(InputEvent::RefreshSubscription);
        }
        if matches_any(&value, &bindings.test_latency) {
            return Self::Input(InputEvent::TestLatency);
        }
        if matches_any(&value, &bindings.toggle_hold) {
            return Self::Input(InputEvent::ToggleHold);
        }
        if matches_any(&value, &bindings.next_sort) {
            return Self::Input(InputEvent::NextSort);
        }
        if matches_any(&value, &bindings.prev_sort) {
            return Self::Input(InputEvent::PrevSort);
        }
        if let Some(digit) = match_tab_goto(&value, &bindings.tab_goto) {
            return Self::Input(InputEvent::TabGoto(digit));
        }

        // Esc is always esc (not worth making configurable)
        if value.code == KC::Esc {
            return Self::Input(InputEvent::Esc);
        }

        // Arrow key navigation stays hardcoded
        match (value.modifiers, value.code) {
            (modi, arrow @ (KC::Left | KC::Right | KC::Up | KC::Down | KC::Enter)) => {
                Event::Input(InputEvent::List(ListEvent {
                    fast: matches!(modi, KM::CONTROL | KM::SHIFT),
                    code: arrow,
                }))
            }
            _ => Self::Input(InputEvent::Other(value)),
        }
    }
}
