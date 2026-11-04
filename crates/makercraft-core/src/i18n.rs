//! MakerCraft Internationalization (i18n) Engine.
//!
//! Follows the fleet standard: typed locale enums, zero-panic string lookups,
//! multi-language translations (EN, DE, FR, ES, JA, KO, ZH),
//! and 3D printing / CNC milling G-code terminology localization.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppLocale {
    #[default]
    En, // English (Global Default)
    De, // German (Precision CNC / German Engineering native)
    Fr, // French
    Es, // Spanish
    Ja, // Japanese (Monozukuri / Precision machining native)
    Ko, // Korean
    Zh, // Chinese (Additive Manufacturing / CNC hub native)
}

impl AppLocale {
    pub fn all() -> &'static [AppLocale] {
        &[
            AppLocale::En,
            AppLocale::De,
            AppLocale::Fr,
            AppLocale::Es,
            AppLocale::Ja,
            AppLocale::Ko,
            AppLocale::Zh,
        ]
    }

    pub fn code(&self) -> &'static str {
        match self {
            AppLocale::En => "en",
            AppLocale::De => "de",
            AppLocale::Fr => "fr",
            AppLocale::Es => "es",
            AppLocale::Ja => "ja",
            AppLocale::Ko => "ko",
            AppLocale::Zh => "zh",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AppLocale::En => "English (US/UK)",
            AppLocale::De => "Deutsch (DE/AT/CH)",
            AppLocale::Fr => "Français (FR)",
            AppLocale::Es => "Español (ES/LATAM)",
            AppLocale::Ja => "日本語 (JA)",
            AppLocale::Ko => "한국어 (KO)",
            AppLocale::Zh => "中文 (ZH)",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            AppLocale::En => "🇬🇧",
            AppLocale::De => "🇩🇪",
            AppLocale::Fr => "🇫🇷",
            AppLocale::Es => "🇪🇸",
            AppLocale::Ja => "🇯🇵",
            AppLocale::Ko => "🇰🇷",
            AppLocale::Zh => "🇨🇳",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum I18nKey {
    // Machine & Slicing
    BedTemperature,
    NozzleTemperature,
    SpindleSpeedRpm,
    FeedRateMmMin,
    GcodeVisualizer,
    SlicingEngine,
    ToolpathSimulation,
    EmergencyStop,
    StatusReady,
}

pub fn t(key: I18nKey, locale: AppLocale) -> &'static str {
    match (key, locale) {
        (I18nKey::BedTemperature, AppLocale::En) => "Heated Bed (°C)",
        (I18nKey::BedTemperature, AppLocale::De) => "Druckbett-Temperatur (°C)",
        (I18nKey::BedTemperature, AppLocale::Fr) => "Lit chauffant (°C)",
        (I18nKey::BedTemperature, AppLocale::Es) => "Cama caliente (°C)",
        (I18nKey::BedTemperature, AppLocale::Ja) => "ヒートベッド温度 (°C)",
        (I18nKey::BedTemperature, AppLocale::Ko) => "히팅 베드 온도 (°C)",
        (I18nKey::BedTemperature, AppLocale::Zh) => "热床加热温度 (°C)",

        (I18nKey::NozzleTemperature, AppLocale::En) => "Extruder Nozzle (°C)",
        (I18nKey::NozzleTemperature, AppLocale::De) => "Extruder-Düse (°C)",
        (I18nKey::NozzleTemperature, AppLocale::Fr) => "Buse d'extrusion (°C)",
        (I18nKey::NozzleTemperature, AppLocale::Es) => "Boquilla de extrusión (°C)",
        (I18nKey::NozzleTemperature, AppLocale::Ja) => "ノズル押出部温度 (°C)",
        (I18nKey::NozzleTemperature, AppLocale::Ko) => "익스트루더 노즐 온도 (°C)",
        (I18nKey::NozzleTemperature, AppLocale::Zh) => "挤出机喷嘴温度 (°C)",

        (I18nKey::SpindleSpeedRpm, AppLocale::En) => "CNC Spindle Speed (RPM)",
        (I18nKey::SpindleSpeedRpm, AppLocale::De) => "CNC-Spindeldrehzahl (RPM)",
        (I18nKey::SpindleSpeedRpm, AppLocale::Fr) => "Vitesse de broche CNC (RPM)",
        (I18nKey::SpindleSpeedRpm, AppLocale::Es) => "Velocidad de husillo CNC (RPM)",
        (I18nKey::SpindleSpeedRpm, AppLocale::Ja) => "CNC 主軸回転速度 (RPM)",
        (I18nKey::SpindleSpeedRpm, AppLocale::Ko) => "CNC 주축 회전수 (RPM)",
        (I18nKey::SpindleSpeedRpm, AppLocale::Zh) => "CNC 主轴转速 (RPM)",

        (I18nKey::FeedRateMmMin, AppLocale::En) => "Feed Rate (mm/min)",
        (I18nKey::FeedRateMmMin, AppLocale::De) => "Vorschubgeschwindigkeit (mm/min)",
        (I18nKey::FeedRateMmMin, AppLocale::Fr) => "Vitesse d'avance (mm/min)",
        (I18nKey::FeedRateMmMin, AppLocale::Es) => "Velocidad de avance (mm/min)",
        (I18nKey::FeedRateMmMin, AppLocale::Ja) => "送り速度 (mm/min)",
        (I18nKey::FeedRateMmMin, AppLocale::Ko) => "이송 속도 (mm/min)",
        (I18nKey::FeedRateMmMin, AppLocale::Zh) => "进给切削速度 (mm/min)",

        (I18nKey::GcodeVisualizer, AppLocale::En) => "G-Code 3D Layer Visualizer",
        (I18nKey::GcodeVisualizer, AppLocale::De) => "G-Code 3D-Schicht-Visualisierer",
        (I18nKey::GcodeVisualizer, AppLocale::Fr) => "Visualiseur 3D de G-Code",
        (I18nKey::GcodeVisualizer, AppLocale::Es) => "Visualizador 3D de G-Code",
        (I18nKey::GcodeVisualizer, AppLocale::Ja) => "G-Code 3D積層ビューアー",
        (I18nKey::GcodeVisualizer, AppLocale::Ko) => "G-Code 3D 적층 뷰어",
        (I18nKey::GcodeVisualizer, AppLocale::Zh) => "G-Code 3D 切片轨迹可视化",

        (I18nKey::SlicingEngine, AppLocale::En) => "Parametric Slicing Engine",
        (I18nKey::SlicingEngine, AppLocale::De) => "Parametrische Slicing-Engine",
        (I18nKey::SlicingEngine, AppLocale::Fr) => "Moteur de tranchage paramétrique",
        (I18nKey::SlicingEngine, AppLocale::Es) => "Motor de laminado paramétrico",
        (I18nKey::SlicingEngine, AppLocale::Ja) => "パラメトリック・スライサーエンジン",
        (I18nKey::SlicingEngine, AppLocale::Ko) => "파라메트릭 슬라이싱 엔진",
        (I18nKey::SlicingEngine, AppLocale::Zh) => "参数化切片内核引擎",

        (I18nKey::ToolpathSimulation, AppLocale::En) => "Toolpath Collision Check",
        (I18nKey::ToolpathSimulation, AppLocale::De) => "Werkzeugbahn-Kollisionsprüfung",
        (I18nKey::ToolpathSimulation, AppLocale::Fr) => "Vérification des collisions d'outils",
        (I18nKey::ToolpathSimulation, AppLocale::Es) => "Comprobación de colisión de trayectoria",
        (I18nKey::ToolpathSimulation, AppLocale::Ja) => "ツールパス干渉・衝突チェック",
        (I18nKey::ToolpathSimulation, AppLocale::Ko) => "공구 경로 간섭 및 충돌 검사",
        (I18nKey::ToolpathSimulation, AppLocale::Zh) => "刀具切削路径干涉与碰撞检测",

        (I18nKey::EmergencyStop, AppLocale::En) => "🛑 EMERGENCY STOP",
        (I18nKey::EmergencyStop, AppLocale::De) => "🛑 NOT-AUS",
        (I18nKey::EmergencyStop, AppLocale::Fr) => "🛑 ARRÊT D'URGENCE",
        (I18nKey::EmergencyStop, AppLocale::Es) => "🛑 PARADA DE EMERGENCIA",
        (I18nKey::EmergencyStop, AppLocale::Ja) => "🛑 非常停止 (E-STOP)",
        (I18nKey::EmergencyStop, AppLocale::Ko) => "🛑 비상 정지",
        (I18nKey::EmergencyStop, AppLocale::Zh) => "🛑 机床紧急急停",

        (I18nKey::StatusReady, AppLocale::En) => "MakerCraft CNC/3D Fabrication HAL Ready",
        (I18nKey::StatusReady, AppLocale::De) => "MakerCraft CNC/3D Fertigungs-HAL bereit",
        (I18nKey::StatusReady, AppLocale::Fr) => "HAL de fabrication 3D/CNC MakerCraft prêt",
        (I18nKey::StatusReady, AppLocale::Es) => "HAL de fabricación 3D/CNC MakerCraft listo",
        (I18nKey::StatusReady, AppLocale::Ja) => "MakerCraft CNC/3D 造形HAL 準備完了",
        (I18nKey::StatusReady, AppLocale::Ko) => "MakerCraft 3D/CNC 가공 HAL 준비완료",
        (I18nKey::StatusReady, AppLocale::Zh) => "MakerCraft 数控与增材制造硬件抽象层就绪",
    }
}
