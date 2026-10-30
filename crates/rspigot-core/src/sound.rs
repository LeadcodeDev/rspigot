use rand::Rng;
use rspigot_domain::Vec3;
use rspigot_domain::player::PlayerId;
use rspigot_entities::{SoundSource, VoicePitch};

use crate::events::ClientEvent;
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const BASE_RANGE: f32 = 16.0;
const PITCH_SPREAD: f32 = 0.2;
const ADULT_PITCH: f32 = 1.0;
const BABY_PITCH: f32 = 1.5;
const BABY_ZOMBIE_VILLAGER_PITCH: f32 = 2.0;
const BAT_PITCH: f32 = 0.95;

pub fn sound_range(volume: f32) -> f64 {
    f64::from(if volume > 1.0 {
        BASE_RANGE * volume
    } else {
        BASE_RANGE
    })
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn play_sound(
        &mut self,
        except: Option<PlayerId>,
        position: Vec3,
        sound: &'static str,
        source: SoundSource,
        volume: f32,
        pitch: f32,
    ) {
        let seed = self.rng.random::<i64>();
        self.push(
            Audience::Nearby {
                position,
                range: sound_range(volume),
                except,
            },
            ClientEvent::Sound {
                sound,
                source,
                position,
                volume,
                pitch,
                seed,
            },
        );
    }

    fn pitch_spread(&mut self) -> f32 {
        let first = self.rng.random::<f32>();
        let second = self.rng.random::<f32>();
        (first - second) * PITCH_SPREAD
    }

    pub(crate) fn voice_pitch(&mut self, rule: VoicePitch, baby: bool) -> f32 {
        match rule {
            VoicePitch::Fixed => ADULT_PITCH,
            VoicePitch::Standard => {
                let spread = self.pitch_spread();
                spread + if baby { BABY_PITCH } else { ADULT_PITCH }
            }
            VoicePitch::Bat => (self.pitch_spread() + ADULT_PITCH) * BAT_PITCH,
            VoicePitch::ZombieVillager => {
                let spread = self.pitch_spread();
                spread
                    + if baby {
                        BABY_ZOMBIE_VILLAGER_PITCH
                    } else {
                        ADULT_PITCH
                    }
            }
            VoicePitch::Parrot => self.pitch_spread() + ADULT_PITCH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loud_sounds_carry_further() {
        assert_eq!(sound_range(0.4), 16.0);
        assert_eq!(sound_range(1.0), 16.0);
        assert_eq!(sound_range(5.0), 80.0);
    }
}
