use rand::Rng;
use rspigot_domain::attribute::{
    ATTACK_DAMAGE, ATTACK_KNOCKBACK, ATTACK_SPEED, KNOCKBACK_RESISTANCE,
};
use rspigot_domain::damage::DamageType;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::{EntityId, Vec3};

use crate::events::{Animation, ClientEvent};
use crate::health::{DamageSource, attributes};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

pub const DEFAULT_KNOCKBACK: f64 = 0.4;
const SPRINT_KNOCKBACK: f32 = 0.5;
const KNOCKBACK_VERTICAL_CAP: f64 = 0.4;
const MIN_KNOCKBACK_DIRECTION: f64 = 1.0e-5;
const RANDOM_DIRECTION_SCALE: f64 = 0.01;
const ATTACK_PARTIAL_TICK: f32 = 0.5;
const FULL_STRENGTH: f32 = 0.9;
const CRITICAL_MULTIPLIER: f32 = 1.5;
const ATTACK_EXHAUSTION: f32 = 0.1;
const ATTACK_RANGE_BUFFER: f64 = 3.0;
const SURVIVAL_REACH: f64 = 3.0;
const CREATIVE_REACH: f64 = 5.0;
const PLAYER_HALF_WIDTH: f64 = 0.3;
const RADIANS_PER_DEGREE: f32 = 0.017_453_292;
const DEGREES_PER_RADIAN: f64 = 57.295_776_367_187_5;

pub fn attack_strength_delay(attack_speed: f64) -> f32 {
    (1.0 / attack_speed * 20.0) as f32
}

pub fn attack_strength_scale(ticker: u32, attack_speed: f64, partial: f32) -> f32 {
    ((ticker as f32 + partial) / attack_strength_delay(attack_speed)).clamp(0.0, 1.0)
}

pub fn base_damage_scale(scale: f32) -> f32 {
    0.2 + scale * scale * 0.8
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strike {
    pub damage: f32,
    pub critical: bool,
    pub sprint_knockback: bool,
}

pub fn strike(base: f32, scale: f32, sprinting: bool, can_critical: bool) -> Option<Strike> {
    let damage = base * base_damage_scale(scale);
    if damage <= 0.0 {
        return None;
    }
    let full = scale > FULL_STRENGTH;
    let critical = full && can_critical && !sprinting;
    Some(Strike {
        damage: if critical {
            damage * CRITICAL_MULTIPLIER
        } else {
            damage
        },
        critical,
        sprint_knockback: sprinting && full,
    })
}

pub fn knockback_velocity(current: Vec3, strength: f64, dx: f64, dz: f64, on_ground: bool) -> Vec3 {
    let length = (dx * dx + dz * dz).sqrt();
    let (nx, nz) = if length < MIN_KNOCKBACK_DIRECTION {
        (0.0, 0.0)
    } else {
        (dx / length * strength, dz / length * strength)
    };
    Vec3::new(
        current.x / 2.0 - nx,
        if on_ground {
            (current.y / 2.0 + strength).min(KNOCKBACK_VERTICAL_CAP)
        } else {
            current.y
        },
        current.z / 2.0 - nz,
    )
}

fn distance_to_box_squared(point: Vec3, center: Vec3, half_width: f64, height: f64) -> f64 {
    let axis = |value: f64, min: f64, max: f64| (min - value).max(0.0).max(value - max);
    let dx = axis(point.x, center.x - half_width, center.x + half_width);
    let dy = axis(point.y, center.y, center.y + height);
    let dz = axis(point.z, center.z - half_width, center.z + half_width);
    dx * dx + dy * dy + dz * dz
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn knockback(&mut self, id: PlayerId, strength: f64, dx: f64, dz: f64) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let strength = strength * (1.0 - attributes(player).get(KNOCKBACK_RESISTANCE));
        if strength <= 0.0 {
            return;
        }
        let (mut dx, mut dz) = (dx, dz);
        while dx * dx + dz * dz < MIN_KNOCKBACK_DIRECTION {
            dx = (self.rng.random::<f64>() - self.rng.random::<f64>()) * RANDOM_DIRECTION_SCALE;
            dz = (self.rng.random::<f64>() - self.rng.random::<f64>()) * RANDOM_DIRECTION_SCALE;
        }
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        player.vitals.velocity =
            knockback_velocity(player.vitals.velocity, strength, dx, dz, player.on_ground);
        player.vitals.sync_velocity = true;
    }

    pub(crate) fn indicate_damage(&mut self, id: PlayerId, dx: f64, dz: f64) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let yaw = (dz.atan2(dx) * DEGREES_PER_RADIAN - f64::from(player.rotation.yaw)) as f32;
        player.vitals.hurt_dir = yaw;
        let entity = player.entity_id;
        self.push(
            Audience::One(id),
            ClientEvent::HurtAnimation { entity, yaw },
        );
    }

    fn in_water(&self, position: Vec3) -> bool {
        self.world
            .chunks
            .block(position.block())
            .is_some_and(|state| state.flags().contains(rspigot_blocks::StateFlags::WATER))
    }

    pub(crate) fn feet_in_water(&self, id: PlayerId) -> bool {
        self.players
            .get(&id)
            .is_some_and(|p| self.in_water(p.position))
    }

    pub(crate) fn handle_attack(&mut self, id: PlayerId, target: EntityId) {
        let Some(attacker) = self.players.get(&id) else {
            return;
        };
        if !attacker.vitals.client_loaded()
            || attacker.game_mode == GameMode::Spectator
            || attacker.vitals.is_dead()
        {
            return;
        }
        if target == attacker.entity_id
            || self.entities.store.get(target).is_some_and(|e| {
                matches!(
                    e.state,
                    rspigot_domain::entity::EntityState::Item(_)
                        | rspigot_domain::entity::EntityState::Falling(_)
                )
            })
        {
            tracing::warn!(
                "Player {} tried to attack an invalid entity",
                attacker.name()
            );
            attacker.link.disconnect(Component::translatable(
                "multiplayer.disconnect.invalid_entity_attacked",
                Vec::new(),
            ));
            return;
        }
        let Some(victim) = self
            .players
            .values()
            .find(|p| p.entity_id == target && !p.vitals.removed)
        else {
            return;
        };
        let reach = if attacker.game_mode == GameMode::Creative {
            CREATIVE_REACH
        } else {
            SURVIVAL_REACH
        } + ATTACK_RANGE_BUFFER;
        let distance = distance_to_box_squared(
            attacker.eye_position(),
            victim.position,
            PLAYER_HALF_WIDTH,
            victim.height(),
        );
        if distance >= reach * reach {
            return;
        }
        let victim_id = victim.id;
        self.attack(id, victim_id);
    }

    fn attack(&mut self, id: PlayerId, victim: PlayerId) {
        let in_water = self.feet_in_water(id);
        let Some(source) = self.source_of(id) else {
            return;
        };
        let Some(attacker) = self.players.get_mut(&id) else {
            return;
        };
        let stats = attributes(attacker);
        let scale = attack_strength_scale(
            attacker.vitals.attack_ticker,
            stats.get(ATTACK_SPEED),
            ATTACK_PARTIAL_TICK,
        );
        attacker.vitals.attack_ticker = 0;
        let can_critical = attacker.vitals.fall_distance > 0.0
            && !attacker.on_ground
            && !in_water
            && !attacker.effects.has("minecraft:blindness");
        let Some(strike) = strike(
            stats.get(ATTACK_DAMAGE) as f32,
            scale,
            attacker.sprinting,
            can_critical,
        ) else {
            return;
        };
        let yaw = attacker.rotation.yaw * RADIANS_PER_DEGREE;
        let attacker_entity = attacker.entity_id;
        let knockback = stats.get(ATTACK_KNOCKBACK) as f32 / 2.0
            + if strike.sprint_knockback {
                SPRINT_KNOCKBACK
            } else {
                0.0
            };
        let source = DamageSource::by(DamageType::PlayerAttack, source);
        if !self.hurt(victim, &source, strike.damage) {
            return;
        }
        if knockback > 0.0 {
            self.knockback(
                victim,
                f64::from(knockback),
                f64::from(yaw.sin()),
                f64::from(-yaw.cos()),
            );
            if let Some(attacker) = self.players.get_mut(&id)
                && attacker.sprinting
            {
                attacker.sprinting = false;
                self.broadcast_metadata(id);
            }
        }
        if let Some(target) = self.players.get_mut(&victim)
            && target.vitals.sync_velocity
        {
            target.vitals.sync_velocity = false;
            let velocity = std::mem::replace(&mut target.vitals.velocity, Vec3::ZERO);
            let entity = target.entity_id;
            self.push(
                Audience::One(victim),
                ClientEvent::EntityVelocity { entity, velocity },
            );
        }
        if strike.critical {
            let target_entity = self.players.get(&victim).map(|p| p.entity_id);
            if let Some(entity) = target_entity {
                let event = ClientEvent::EntityAnimation {
                    entity,
                    animation: Animation::CriticalHit,
                };
                self.push(Audience::One(id), event.clone());
                self.push(Audience::Trackers(attacker_entity), event);
            }
        }
        self.exhaust(id, ATTACK_EXHAUSTION);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fist_cooldown_is_five_ticks_and_sword_cooldown_twelve_and_a_half() {
        assert_eq!(attack_strength_delay(4.0), 5.0);
        assert!((attack_strength_delay(1.6) - 12.5).abs() < 1e-5);
        assert_eq!(attack_strength_scale(0, 4.0, 0.5), 0.1);
        assert_eq!(attack_strength_scale(5, 4.0, 0.5), 1.0);
        assert!((attack_strength_scale(6, 1.6, 0.5) - 0.52).abs() < 1e-5);
        assert_eq!(attack_strength_scale(100, 1.6, 0.5), 1.0);
    }

    #[test]
    fn spamming_scales_damage_down_to_a_fifth() {
        assert_eq!(base_damage_scale(1.0), 1.0);
        assert!((base_damage_scale(0.0) - 0.2).abs() < 1e-6);
        let weak = strike(7.0, 0.1, false, true).unwrap();
        assert!((weak.damage - 7.0 * (0.2 + 0.01 * 0.8)).abs() < 1e-5);
        assert!(!weak.critical);
    }

    #[test]
    fn critical_hits_need_full_strength_and_no_sprint() {
        let critical = strike(7.0, 1.0, false, true).unwrap();
        assert_eq!(critical.damage, 10.5);
        assert!(critical.critical);
        assert!(!critical.sprint_knockback);
        let sprinting = strike(7.0, 1.0, true, true).unwrap();
        assert_eq!(sprinting.damage, 7.0);
        assert!(!sprinting.critical);
        assert!(sprinting.sprint_knockback);
        let grounded = strike(1.0, 1.0, false, false).unwrap();
        assert_eq!(grounded.damage, 1.0);
        assert!(strike(0.0, 1.0, false, true).is_none());
    }

    #[test]
    fn knockback_pushes_away_and_lifts_grounded_targets() {
        let pushed = knockback_velocity(Vec3::ZERO, 0.4, 1.0, 0.0, true);
        assert_eq!(pushed, Vec3::new(-0.4, 0.4, 0.0));
        let airborne = knockback_velocity(Vec3::new(0.0, -0.3, 0.0), 0.4, 0.0, 2.0, false);
        assert_eq!(airborne, Vec3::new(0.0, -0.3, -0.4));
        let stacked = knockback_velocity(pushed, 0.5, 0.0, -1.0, true);
        assert_eq!(stacked, Vec3::new(-0.2, 0.4, 0.5));
    }

    #[test]
    fn reach_is_measured_to_the_target_box() {
        let target = Vec3::new(0.0, 64.0, 0.0);
        assert_eq!(
            distance_to_box_squared(Vec3::new(0.0, 65.0, 0.0), target, 0.3, 1.8),
            0.0
        );
        let far = distance_to_box_squared(Vec3::new(3.3, 65.0, 0.0), target, 0.3, 1.8);
        assert!((far - 9.0).abs() < 1e-9);
    }
}
