//! Rendu des explosions et interceptions via le trait `Host`.
//!
//! Aucune API Horizon n'est appelée ici. Le module `glue.rs` implémente
//! `Host` avec les fonctions vérifiées (particules, sons, secousse caméra).

use crate::civ_core::math::Vec3;
use crate::civ_core::missile::{DamageReport, ExplosionEffectSpec, MissileInstance};

use crate::host::Host;

/// Type d'explosion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplosionKind {
    Conventional,
    Nuclear,
    Chemical,
    Emp,
    Incendiary,
    Orbital,
}

/// Contexte visuel d'un impact.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplosionContext {
    pub position: Vec3,
    pub radius: f64,
    pub kind: ExplosionKind,
    pub spec: Option<ExplosionEffectSpec>,
}

impl ExplosionContext {
    /// Construit un contexte avec les valeurs par défaut de l'adaptateur.
    pub fn new(position: Vec3, radius: f64, kind: ExplosionKind) -> Self {
        Self {
            position,
            radius,
            kind,
            spec: None,
        }
    }

    /// Attache les effets déclarés par le type de missile.
    pub fn with_spec(mut self, spec: ExplosionEffectSpec) -> Self {
        self.spec = Some(spec);
        self
    }
}

/// Stratégie de rendu des effets de missile.
pub trait ExplosionEffects {
    /// Impact d'un missile.
    fn impact(&mut self, host: &mut dyn Host, context: &ExplosionContext, report: &DamageReport);
    /// Interception avant impact.
    fn intercepted(
        &mut self,
        host: &mut dyn Host,
        context: &ExplosionContext,
        missile: &MissileInstance,
    );
}

/// Implémentation par défaut, configurable par données.
#[derive(Clone, Debug)]
pub struct DefaultExplosionEffects {
    pub fallback_particle: String,
    pub fallback_sound: String,
    pub shake_intensity: f32,
    pub shake_ticks: u32,
}

impl Default for DefaultExplosionEffects {
    fn default() -> Self {
        Self {
            fallback_particle: "minecraft:explosion".to_owned(),
            fallback_sound: "minecraft:entity.generic.explode".to_owned(),
            shake_intensity: 0.6,
            shake_ticks: 20,
        }
    }
}

impl DefaultExplosionEffects {
    /// Crée une stratégie avec des ressources visuelles explicites.
    pub fn new(
        fallback_particle: impl Into<String>,
        fallback_sound: impl Into<String>,
        shake_intensity: f32,
        shake_ticks: u32,
    ) -> Self {
        Self {
            fallback_particle: fallback_particle.into(),
            fallback_sound: fallback_sound.into(),
            shake_intensity,
            shake_ticks,
        }
    }

    fn particle<'a>(&'a self, context: &'a ExplosionContext) -> &'a str {
        context
            .spec
            .as_ref()
            .and_then(|spec| spec.particle.as_deref())
            .unwrap_or(&self.fallback_particle)
    }

    fn sound<'a>(&'a self, context: &'a ExplosionContext) -> &'a str {
        context
            .spec
            .as_ref()
            .and_then(|spec| spec.sound.as_deref())
            .unwrap_or(&self.fallback_sound)
    }
}

impl ExplosionEffects for DefaultExplosionEffects {
    fn impact(&mut self, host: &mut dyn Host, context: &ExplosionContext, report: &DamageReport) {
        let particle = self.particle(context).to_owned();
        let sound = self.sound(context).to_owned();
        host.spawn_particle(&particle, context.position);
        host.play_sound(&sound, context.position);
        if context.radius > 0.0 {
            host.camera_shake(self.shake_intensity, self.shake_ticks.max(1));
        }
        let kind = match context.kind {
            ExplosionKind::Conventional => "conventionnelle",
            ExplosionKind::Nuclear => "nucléaire",
            ExplosionKind::Chemical => "chimique",
            ExplosionKind::Emp => "EMP",
            ExplosionKind::Incendiary => "incendiaire",
            ExplosionKind::Orbital => "orbitale",
        };
        host.info(&format!(
            "Impact {kind} : {} victime(s), dégâts d'infrastructure {:.1}%, escalade {:.1}%",
            report.casualties, report.infrastructure_damage, report.escalation
        ));
        if context
            .spec
            .as_ref()
            .is_some_and(|spec| spec.orbital_laser_hook)
        {
            host.warn(
                "Orbital Laser : rendu visuel demandé ; aucune frappe coordonnée n'est exposée par l'API vérifiée.",
            );
        }
    }

    fn intercepted(
        &mut self,
        host: &mut dyn Host,
        context: &ExplosionContext,
        missile: &MissileInstance,
    ) {
        host.spawn_particle("minecraft:smoke", context.position);
        host.play_sound("minecraft:entity.firework_rocket.launch", context.position);
        host.info(&format!(
            "Missile {} intercepté avant impact sur {}.",
            missile.id, missile.target.province
        ));
    }
}

/// Stratégie silencieuse pour les tests et les serveurs dédiés.
#[derive(Clone, Copy, Debug, Default)]
pub struct SilentExplosionEffects;

impl ExplosionEffects for SilentExplosionEffects {
    fn impact(
        &mut self,
        _host: &mut dyn Host,
        _context: &ExplosionContext,
        _report: &DamageReport,
    ) {
    }

    fn intercepted(
        &mut self,
        _host: &mut dyn Host,
        _context: &ExplosionContext,
        _missile: &MissileInstance,
    ) {
    }
}
