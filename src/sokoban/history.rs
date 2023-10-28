use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Default)]
pub struct HistoryPlugin<C: Component + Clone> {
    phantom: PhantomData<C>,
}

impl<C: Component + Clone> Plugin for HistoryPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_event::<HistoryEvent>();
        app.add_systems(
            Update,
            handle_history_commands::<C>.in_set(HandleHistoryEvents),
        );
    }
}

#[derive(Event)]
pub enum HistoryEvent {
    Record,
    Rewind,
    Reset,
}

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct HandleHistoryEvents;

#[derive(Component, Clone, Default, Deref, DerefMut, Reflect)]
pub struct History<C: Component + Clone>(Vec<C>);

fn handle_history_commands<C: Component + Clone>(
    mut history_query: Query<(&mut History<C>, &mut C)>,
    mut history_events: EventReader<HistoryEvent>,
) {
    for ev in history_events.iter() {
        match ev {
            HistoryEvent::Record => {
                for (mut history, component) in history_query.iter_mut() {
                    history.push(component.clone());
                }
            }
            HistoryEvent::Rewind => {
                for (mut history, mut component) in history_query.iter_mut() {
                    if let Some(prev) = history.pop() {
                        *component = prev;
                    }
                }
            }
            HistoryEvent::Reset => {
                for (mut history, mut component) in history_query.iter_mut() {
                    if let Some(first) = history.first() {
                        let first = first.clone();
                        history.push(component.clone());
                        *component = first;
                    }
                }
            }
        }
    }
}
