//! Embedded MakerCraft skills (`skills/*/SKILL.md`) for MCP resources,
//! CLI export, in-app Help, and local-first AI chat preprompting.

/// One bundled skill compiled into the binary.
#[derive(Debug, Clone, Copy)]
pub struct BundledSkill {
    pub name: &'static str,
    pub description: &'static str,
    pub skill_md: &'static str,
}

pub const SKILLS: &[BundledSkill] = &[
    BundledSkill {
        name: "makercraft-workflow",
        description: "Core MakerCraft workflow for G-code parsing, safety linting, 2D toolpath preview, and CLI/MCP bench operations.",
        skill_md: include_str!("../../../skills/makercraft-workflow/SKILL.md"),
    },
    BundledSkill {
        name: "makercraft-klipper-printer",
        description: "Operating Klipper and Moonraker 3D printers safely, checking temperatures, bed envelope, and first-layer diagnostics.",
        skill_md: include_str!("../../../skills/makercraft-klipper-printer/SKILL.md"),
    },
    BundledSkill {
        name: "makercraft-grbl-cnc",
        description: "Operating 3018-class GRBL 1.1h CNC routers, work coordinates, plunge safety, spindle RPM, and endmill selection.",
        skill_md: include_str!("../../../skills/makercraft-grbl-cnc/SKILL.md"),
    },
    BundledSkill {
        name: "makercraft-oscilloscope",
        description: "Probing circuits with SCPI/USBTMC oscilloscopes, ground-clip safety, trigger setup, and stepper/PWM diagnostics.",
        skill_md: include_str!("../../../skills/makercraft-oscilloscope/SKILL.md"),
    },
    BundledSkill {
        name: "makercraft-logic-analyzer",
        description: "Capturing digital buses with sigrok/SUMP logic analyzers, sample-rate rules, and I2C/SPI/UART/Step-Dir decoding.",
        skill_md: include_str!("../../../skills/makercraft-logic-analyzer/SKILL.md"),
    },
    BundledSkill {
        name: "makercraft-bus-pirate",
        description: "Using Bus Pirate v4 at 3.3V/5V for I2C/SPI/UART probing, pull-up resistors, and pin voltage checks.",
        skill_md: include_str!("../../../skills/makercraft-bus-pirate/SKILL.md"),
    },
    BundledSkill {
        name: "makercraft-vienna-makerspaces",
        description: "Complete reference and how-to for Vienna makerspaces (Metalab, Happylab, Maker Austria), equipment, briefings, and etiquette.",
        skill_md: include_str!("../../../skills/makercraft-vienna-makerspaces/SKILL.md"),
    },
    BundledSkill {
        name: "makercraft-buying-guide",
        description: "Practical buying advice for 3D printers, desktop CNCs, oscilloscopes, logic analyzers, multimeters, and debug probes.",
        skill_md: include_str!("../../../skills/makercraft-buying-guide/SKILL.md"),
    },
];

/// Return all bundled skills.
#[must_use]
pub fn skills_list() -> &'static [BundledSkill] {
    SKILLS
}

/// Look up a bundled skill by exact folder name.
#[must_use]
pub fn skill_by_name(name: &str) -> Option<&'static BundledSkill> {
    SKILLS.iter().find(|s| s.name == name)
}

/// Look up a bundled skill by MCP `skill://<name>/SKILL.md` URI.
#[must_use]
pub fn skill_by_uri(uri: &str) -> Option<&'static BundledSkill> {
    let trimmed = uri.strip_prefix("skill://")?.strip_suffix("/SKILL.md")?;
    skill_by_name(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_skill_folders_are_bundled_and_within_limits() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let skills_dir = std::path::Path::new(manifest_dir).join("../../skills");
        let entries_res = std::fs::read_dir(&skills_dir);
        assert!(
            entries_res.is_ok(),
            "skills directory missing at {}",
            skills_dir.display()
        );
        let Ok(entries) = entries_res else {
            return;
        };

        let mut folder_count = 0usize;
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                folder_count += 1;
                let folder_name = entry.file_name().to_string_lossy().to_string();
                let found = skill_by_name(&folder_name);
                assert!(
                    found.is_some(),
                    "skill folder '{folder_name}' is not listed in SKILLS"
                );
            }
        }
        assert_eq!(folder_count, SKILLS.len());

        let mut total_bytes = 0usize;
        for skill in SKILLS {
            assert!(
                skill.skill_md.len() <= 12 * 1024,
                "{} exceeds 12 KB",
                skill.name
            );
            assert!(
                skill.description.len() <= 200,
                "{} description exceeds 200 chars",
                skill.name
            );
            assert!(
                skill.skill_md.starts_with("---\n"),
                "{} missing YAML frontmatter",
                skill.name
            );
            assert!(
                !skill.skill_md.contains('\r'),
                "{} contains CRLF; expected LF",
                skill.name
            );
            total_bytes += skill.skill_md.len();
        }
        assert!(
            total_bytes <= 40 * 1024,
            "total skills {total_bytes} > 40 KB"
        );
    }
}
