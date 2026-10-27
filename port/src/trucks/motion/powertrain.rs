//! Powertrain inputs and its complete proposed/retained result. The fleet
//! commits stock/history/fractions immediately, but speed and heading remain
//! proposals until blocked response and sequential pair contacts have finished.
use super::{
    EffectSprite, MotionEffect, MotionTables, NITRO_SOUND_COMMAND, NITRO_SPEED, Truck, Upgrades,
    set_low,
};
use crate::{controls::ControllerWord, trucks::nitro::NitroStock};

#[derive(Clone, Copy)]
pub(super) struct PowertrainInput {
    controller: ControllerWord,
    upgrades: Upgrades,
    screen_x: u16,
    screen_y: u8,
    contact_marker: u8,
}
impl From<&Truck> for PowertrainInput {
    fn from(truck: &Truck) -> Self {
        Self {
            controller: truck.controller,
            upgrades: truck.upgrades,
            screen_x: truck.screen_x,
            screen_y: truck.screen_y,
            contact_marker: truck.contact_marker,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct PowertrainState {
    pub(super) speed: u16,
    pub(super) heading: u16,
    acceleration_fraction: u16,
    steering_fraction: u8,
    nitro: NitroStock,
    nitro_button_history: u8,
    nitro_animation: u8,
}
impl From<&Truck> for PowertrainState {
    fn from(truck: &Truck) -> Self {
        Self {
            speed: truck.speed,
            heading: truck.heading,
            acceleration_fraction: truck.acceleration_fraction,
            steering_fraction: truck.steering_fraction,
            nitro: truck.nitro,
            nitro_button_history: truck.nitro_button_history,
            nitro_animation: truck.nitro_animation,
        }
    }
}
impl PowertrainState {
    /// Exhaustive destructuring makes every retained output a deliberate choice.
    /// Speed/heading must NOT replace committed fields in the fleet here: pair
    /// geometry and terrain rejection still read the old current-truck values.
    pub(super) fn commit_retained(&self, truck: &mut Truck) {
        let Self {
            speed: _,
            heading: _,
            acceleration_fraction,
            steering_fraction,
            nitro,
            nitro_button_history,
            nitro_animation,
        } = *self;
        truck.acceleration_fraction = acceleration_fraction;
        truck.steering_fraction = steering_fraction;
        truck.nitro = nitro;
        truck.nitro_button_history = nitro_button_history;
        truck.nitro_animation = nitro_animation;
    }
    /// Isolated domain entries intentionally commit their speed/heading too.
    /// Fleet service instead calls commit_retained and keeps the proposal local.
    pub(super) fn commit_all(self, truck: &mut Truck) {
        self.commit_retained(truck);
        truck.speed = self.speed;
        truck.heading = self.heading;
    }
    pub(super) fn steer(&mut self, input: &PowertrainInput, tables: &MotionTables) {
        use crate::controls::{BUTTON_L, BUTTON_LEFT, BUTTON_R, BUTTON_RIGHT};
        const DIRECTION_STEP: u16 = 8;
        const SHOULDER_STEP: u16 = 4;
        const HEADING_MASK: u16 = 0xff;
        let (fraction, carry) = self
            .steering_fraction
            .overflowing_add(tables.steering_rates[usize::from(input.upgrades.steering)]);
        self.steering_fraction = fraction;
        if !carry {
            return;
        }
        for (button, delta) in [
            (BUTTON_RIGHT, DIRECTION_STEP.wrapping_neg()),
            (BUTTON_R, SHOULDER_STEP.wrapping_neg()),
            (BUTTON_LEFT, DIRECTION_STEP),
            (BUTTON_L, SHOULDER_STEP),
        ] {
            if input.controller.0 & button != 0 {
                self.heading = self.heading.wrapping_add(delta) & HEADING_MASK;
            }
        }
    }

    pub(super) fn use_nitro(
        &mut self,
        input: &PowertrainInput,
        prepared: bool,
        active_humans: u8,
    ) -> Vec<MotionEffect> {
        use crate::controls::BUTTON_Y;
        const CURRENT_PRESS: u8 = 0x80;
        const PREVIOUS_PRESS: u8 = 0x40;
        if prepared {
            return Vec::new();
        }
        let held = input.controller.0 & BUTTON_Y != 0;
        self.nitro_button_history =
            (self.nitro_button_history >> 1) | if held { CURRENT_PRESS } else { 0 };
        if self.nitro_button_history & (CURRENT_PRESS | PREVIOUS_PRESS) != CURRENT_PRESS
            || !self.nitro.consume()
        {
            return Vec::new();
        }
        set_low(&mut self.speed, NITRO_SPEED);
        let mut effects = Vec::new();
        if input.screen_x <= u16::from(u8::MAX) {
            self.nitro_animation = 1;
            effects.push(MotionEffect::NitroSprite(EffectSprite {
                x: input.screen_x as u8,
                y: input.screen_y,
            }));
        }
        if active_humans != 0 {
            effects.push(MotionEffect::Sound(NITRO_SOUND_COMMAND));
        }
        effects
    }

    pub(super) fn accelerate(
        &mut self,
        input: &PowertrainInput,
        tables: &MotionTables,
        human: bool,
        animation: u8,
    ) {
        use crate::controls::BUTTON_B;
        const ZERO_UPGRADE_COMPUTER_LIMIT: usize = 8;
        let accumulated = self
            .acceleration_fraction
            .wrapping_add(tables.acceleration_steps[usize::from(input.upgrades.acceleration)]);
        self.acceleration_fraction = u16::from(accumulated as u8);
        let step = (accumulated >> u8::BITS) as u8;
        let old = self.speed as u8;
        let new = if input.controller.0 & BUTTON_B == 0 || input.contact_marker == 0 {
            match (old as i8).cmp(&0) {
                std::cmp::Ordering::Less => old.wrapping_add(1),
                std::cmp::Ordering::Greater => old - 1,
                std::cmp::Ordering::Equal => old,
            }
        } else if (old as i8) < 0 {
            old.wrapping_add(step)
        } else {
            let selector = if input.upgrades.speed == 0 && !human {
                ZERO_UPGRADE_COMPUTER_LIMIT
            } else {
                usize::from(input.upgrades.speed)
            };
            let limit = tables.speed_limits[selector];
            if old < limit {
                let (candidate, overflow) = (old as i8).overflowing_add(step as i8);
                if overflow {
                    return;
                }
                if (candidate as u8) < limit {
                    set_low(&mut self.speed, candidate as u8);
                    return;
                }
            }
            if animation & 1 == 0 {
                old
            } else {
                old.wrapping_sub(1)
            }
        };
        set_low(&mut self.speed, new);
    }
}
pub(super) struct PowertrainOutcome {
    pub(super) state: PowertrainState,
    pub(super) effects: Vec<MotionEffect>,
}
/// Preserve steering→nitro→acceleration and the observable source-width writes.
/// This reads only actual powertrain dependencies, never clones wheels, world,
/// projection or contact state into a second mutable vehicle authority.
pub(super) fn propose(
    truck: &Truck,
    tables: &MotionTables,
    human: bool,
    active_humans: u8,
    prepared: bool,
    animation: u8,
) -> PowertrainOutcome {
    let input = PowertrainInput::from(truck);
    let mut state = PowertrainState::from(truck);
    state.steer(&input, tables);
    let effects = state.use_nitro(&input, prepared, active_humans);
    state.accelerate(&input, tables, human, animation);
    PowertrainOutcome { state, effects }
}
