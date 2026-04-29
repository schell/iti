//! Classic Mac OS system icons as PNG images.
//!
//! Provides access to 149 authentic Mac OS 9 icons across 6 categories:
//! System (41), Applications (22), ControlPanel (34), ControlStrip (12),
//! Folder (24), MenuBar (16).
//!
//! Supports two loading modes:
//!
//! - **Embedded** (with `embed-assets` feature): Icons compile into the WASM
//!   binary and are served via Blob URLs. No network requests.
//! - **URL** (default): Icons load from `/icons-classic/{category}/{icon}.png`.
//!   Requires Trunk to copy assets or manual CDN/static hosting setup.
//!
//! # Example
//!
//! ```ignore
//! use iti::components::icon_classic::{IconClassic, IconClassicGlyph, SystemIcon};
//! use mogwai::web::prelude::*;
//!
//! let icon = IconClassic::<Web>::new(
//!     IconClassicGlyph::System(SystemIcon::HardDrive)
//! );
//! mogwai::web::body().append_child(&icon);
//! ```

use mogwai::prelude::*;

/// System icons (HardDrive, SystemFolder, Trash, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SystemIcon {
    BlankFile,
    Clipboard,
    ColorProfile,
    ColorSw1500,
    ColorSw2500,
    ColorSwPro,
    Csw6000Series,
    DriveSetup,
    Edit,
    FloppyDisk,
    FolderPanels,
    FontFile,
    FontSuitcase,
    HardDrive,
    HardDriveExternal,
    HardDriveFinder,
    HardDriveShared,
    HardDriveSharedFinder,
    Help,
    InternetBrowse,
    InternetSearch,
    InternetSetup,
    ItunesPlaylist,
    ItunesPlugin,
    LanguageFile,
    Log,
    MapTcpFile,
    NotepadFile,
    Pdf,
    Question,
    QuicktimeMovie,
    RegisterWithApple,
    Screen,
    SetupAssistant,
    SoundSettings,
    SystemFolder,
    TeachText,
    TextFile,
    TrashEmpty,
    TrashFull,
    UrlAccess,
}

impl SystemIcon {
    pub fn filename(&self) -> &str {
        match self {
            SystemIcon::BlankFile => "Blank file.png",
            SystemIcon::Clipboard => "Clipboard.png",
            SystemIcon::ColorProfile => "Color profile.png",
            SystemIcon::ColorSw1500 => "Color SW 1500.png",
            SystemIcon::ColorSw2500 => "Color SW 2500.png",
            SystemIcon::ColorSwPro => "Color SW Pro.png",
            SystemIcon::Csw6000Series => "CSW 6000 Series.png",
            SystemIcon::DriveSetup => "Drive setup.png",
            SystemIcon::Edit => "Edit.png",
            SystemIcon::FloppyDisk => "Floppy disk.png",
            SystemIcon::FolderPanels => "Folder panels.png",
            SystemIcon::FontFile => "Font file.png",
            SystemIcon::FontSuitcase => "Font suitcase.png",
            SystemIcon::HardDrive => "Hard drive.png",
            SystemIcon::HardDriveExternal => "Hard drive (external).png",
            SystemIcon::HardDriveFinder => "Hard drive (finder).png",
            SystemIcon::HardDriveShared => "Hard drive (Shared).png",
            SystemIcon::HardDriveSharedFinder => "Hard drive (shared finder).png",
            SystemIcon::Help => "Help.png",
            SystemIcon::InternetBrowse => "Internet Browse.png",
            SystemIcon::InternetSearch => "Internet search.png",
            SystemIcon::InternetSetup => "Internet setup.png",
            SystemIcon::ItunesPlaylist => "iTunes playlist.png",
            SystemIcon::ItunesPlugin => "iTunes plugin.png",
            SystemIcon::LanguageFile => "Language file.png",
            SystemIcon::Log => "Log.png",
            SystemIcon::MapTcpFile => "Map tcp file.png",
            SystemIcon::NotepadFile => "Notepad file.png",
            SystemIcon::Pdf => "PDF.png",
            SystemIcon::Question => "Question.png",
            SystemIcon::QuicktimeMovie => "Quicktime movie.png",
            SystemIcon::RegisterWithApple => "Register with Apple.png",
            SystemIcon::Screen => "Screen.png",
            SystemIcon::SetupAssistant => "Setup assistant.png",
            SystemIcon::SoundSettings => "Sound settings.png",
            SystemIcon::SystemFolder => "System folder.png",
            SystemIcon::TeachText => "Teach text.png",
            SystemIcon::TextFile => "Text file.png",
            SystemIcon::TrashEmpty => "Trash (Empty).png",
            SystemIcon::TrashFull => "Trash (Full).png",
            SystemIcon::UrlAccess => "URL Access.png",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            SystemIcon::BlankFile => "Blank File",
            SystemIcon::Clipboard => "Clipboard",
            SystemIcon::ColorProfile => "Color Profile",
            SystemIcon::ColorSw1500 => "Color SW 1500",
            SystemIcon::ColorSw2500 => "Color SW 2500",
            SystemIcon::ColorSwPro => "Color SW Pro",
            SystemIcon::Csw6000Series => "CSW 6000 Series",
            SystemIcon::DriveSetup => "Drive Setup",
            SystemIcon::Edit => "Edit",
            SystemIcon::FloppyDisk => "Floppy Disk",
            SystemIcon::FolderPanels => "Folder Panels",
            SystemIcon::FontFile => "Font File",
            SystemIcon::FontSuitcase => "Font Suitcase",
            SystemIcon::HardDrive => "Hard Drive",
            SystemIcon::HardDriveExternal => "Hard Drive (External)",
            SystemIcon::HardDriveFinder => "Hard Drive (Finder)",
            SystemIcon::HardDriveShared => "Hard Drive (Shared)",
            SystemIcon::HardDriveSharedFinder => "Hard Drive (Shared Finder)",
            SystemIcon::Help => "Help",
            SystemIcon::InternetBrowse => "Internet Browse",
            SystemIcon::InternetSearch => "Internet Search",
            SystemIcon::InternetSetup => "Internet Setup",
            SystemIcon::ItunesPlaylist => "iTunes Playlist",
            SystemIcon::ItunesPlugin => "iTunes Plugin",
            SystemIcon::LanguageFile => "Language File",
            SystemIcon::Log => "Log",
            SystemIcon::MapTcpFile => "Map TCP File",
            SystemIcon::NotepadFile => "Notepad File",
            SystemIcon::Pdf => "PDF",
            SystemIcon::Question => "Question",
            SystemIcon::QuicktimeMovie => "QuickTime Movie",
            SystemIcon::RegisterWithApple => "Register with Apple",
            SystemIcon::Screen => "Screen",
            SystemIcon::SetupAssistant => "Setup Assistant",
            SystemIcon::SoundSettings => "Sound Settings",
            SystemIcon::SystemFolder => "System Folder",
            SystemIcon::TeachText => "TeachText",
            SystemIcon::TextFile => "Text File",
            SystemIcon::TrashEmpty => "Trash (Empty)",
            SystemIcon::TrashFull => "Trash (Full)",
            SystemIcon::UrlAccess => "URL Access",
        }
    }

    pub const ALL: [SystemIcon; 41] = [
        SystemIcon::BlankFile,
        SystemIcon::Clipboard,
        SystemIcon::ColorProfile,
        SystemIcon::ColorSw1500,
        SystemIcon::ColorSw2500,
        SystemIcon::ColorSwPro,
        SystemIcon::Csw6000Series,
        SystemIcon::DriveSetup,
        SystemIcon::Edit,
        SystemIcon::FloppyDisk,
        SystemIcon::FolderPanels,
        SystemIcon::FontFile,
        SystemIcon::FontSuitcase,
        SystemIcon::HardDrive,
        SystemIcon::HardDriveExternal,
        SystemIcon::HardDriveFinder,
        SystemIcon::HardDriveShared,
        SystemIcon::HardDriveSharedFinder,
        SystemIcon::Help,
        SystemIcon::InternetBrowse,
        SystemIcon::InternetSearch,
        SystemIcon::InternetSetup,
        SystemIcon::ItunesPlaylist,
        SystemIcon::ItunesPlugin,
        SystemIcon::LanguageFile,
        SystemIcon::Log,
        SystemIcon::MapTcpFile,
        SystemIcon::NotepadFile,
        SystemIcon::Pdf,
        SystemIcon::Question,
        SystemIcon::QuicktimeMovie,
        SystemIcon::RegisterWithApple,
        SystemIcon::Screen,
        SystemIcon::SetupAssistant,
        SystemIcon::SoundSettings,
        SystemIcon::SystemFolder,
        SystemIcon::TeachText,
        SystemIcon::TextFile,
        SystemIcon::TrashEmpty,
        SystemIcon::TrashFull,
        SystemIcon::UrlAccess,
    ];
}

/// Application icons (Calculator, iTunes, Mail, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApplicationsIcon {
    AdobeIllustrator55,
    AdobePhotoshop50,
    AppleFileecurity,
    AppleFmRadio,
    AppleSharePrep,
    AppleVerifier,
    Calculator,
    DiskCopy,
    DiskFirstAid,
    GraphingCalculator,
    Itunes,
    KeyCaps,
    Mail,
    MicrosoftInternetExplorer,
    Notepad,
    QuicktimePlayer,
    Scrapbook,
    ScriptEditor,
    Sherlock20,
    Stickies,
    Stuffitexpander,
    Website,
}

impl ApplicationsIcon {
    pub fn filename(&self) -> &str {
        match self {
            ApplicationsIcon::AdobeIllustrator55 => "Adobe Illustrator 5.5.png",
            ApplicationsIcon::AdobePhotoshop50 => "Adobe Photoshop 5.0.png",
            ApplicationsIcon::AppleFileecurity => "Apple File security.png",
            ApplicationsIcon::AppleFmRadio => "Apple FM radio.png",
            ApplicationsIcon::AppleSharePrep => "Apple Share Prep.png",
            ApplicationsIcon::AppleVerifier => "Apple verifier.png",
            ApplicationsIcon::Calculator => "Calculator.png",
            ApplicationsIcon::DiskCopy => "Disk copy.png",
            ApplicationsIcon::DiskFirstAid => "Disk first aid.png",
            ApplicationsIcon::GraphingCalculator => "Graphing calculator.png",
            ApplicationsIcon::Itunes => "iTunes.png",
            ApplicationsIcon::KeyCaps => "Key caps.png",
            ApplicationsIcon::Mail => "Mail.png",
            ApplicationsIcon::MicrosoftInternetExplorer => "Microsoft Internet Explorer.png",
            ApplicationsIcon::Notepad => "Notepad.png",
            ApplicationsIcon::QuicktimePlayer => "Quicktime Player.png",
            ApplicationsIcon::Scrapbook => "Scrapbook.png",
            ApplicationsIcon::ScriptEditor => "Script editor.png",
            ApplicationsIcon::Sherlock20 => "Sherlock 2.0.png",
            ApplicationsIcon::Stickies => "Stickies.png",
            ApplicationsIcon::Stuffitexpander => "Stuffit expander.png",
            ApplicationsIcon::Website => "Website.png",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ApplicationsIcon::AdobeIllustrator55 => "Adobe Illustrator 5.5",
            ApplicationsIcon::AdobePhotoshop50 => "Adobe Photoshop 5.0",
            ApplicationsIcon::AppleFileecurity => "Apple File Security",
            ApplicationsIcon::AppleFmRadio => "Apple FM Radio",
            ApplicationsIcon::AppleSharePrep => "Apple Share Prep",
            ApplicationsIcon::AppleVerifier => "Apple Verifier",
            ApplicationsIcon::Calculator => "Calculator",
            ApplicationsIcon::DiskCopy => "Disk Copy",
            ApplicationsIcon::DiskFirstAid => "Disk First Aid",
            ApplicationsIcon::GraphingCalculator => "Graphing Calculator",
            ApplicationsIcon::Itunes => "iTunes",
            ApplicationsIcon::KeyCaps => "Key Caps",
            ApplicationsIcon::Mail => "Mail",
            ApplicationsIcon::MicrosoftInternetExplorer => "Microsoft Internet Explorer",
            ApplicationsIcon::Notepad => "Notepad",
            ApplicationsIcon::QuicktimePlayer => "QuickTime Player",
            ApplicationsIcon::Scrapbook => "Scrapbook",
            ApplicationsIcon::ScriptEditor => "Script Editor",
            ApplicationsIcon::Sherlock20 => "Sherlock 2.0",
            ApplicationsIcon::Stickies => "Stickies",
            ApplicationsIcon::Stuffitexpander => "StuffIt Expander",
            ApplicationsIcon::Website => "Website",
        }
    }

    pub const ALL: [ApplicationsIcon; 22] = [
        ApplicationsIcon::AdobeIllustrator55,
        ApplicationsIcon::AdobePhotoshop50,
        ApplicationsIcon::AppleFileecurity,
        ApplicationsIcon::AppleFmRadio,
        ApplicationsIcon::AppleSharePrep,
        ApplicationsIcon::AppleVerifier,
        ApplicationsIcon::Calculator,
        ApplicationsIcon::DiskCopy,
        ApplicationsIcon::DiskFirstAid,
        ApplicationsIcon::GraphingCalculator,
        ApplicationsIcon::Itunes,
        ApplicationsIcon::KeyCaps,
        ApplicationsIcon::Mail,
        ApplicationsIcon::MicrosoftInternetExplorer,
        ApplicationsIcon::Notepad,
        ApplicationsIcon::QuicktimePlayer,
        ApplicationsIcon::Scrapbook,
        ApplicationsIcon::ScriptEditor,
        ApplicationsIcon::Sherlock20,
        ApplicationsIcon::Stickies,
        ApplicationsIcon::Stuffitexpander,
        ApplicationsIcon::Website,
    ];
}

/// Control Panel icons (Appearance, Network, Sound, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ControlPanelIcon {
    AdobeGamma,
    Appearance,
    AppleMenuOptions,
    Appletalk,
    Atm,
    Colorsync,
    ControlStrip,
    DateAndTime,
    DialAssist,
    EnergySaver,
    ExtensionsManager,
    FileExchange,
    FileSharing,
    GeneralControls,
    Internet,
    Keyboard,
    KeychainAccess,
    Launcher,
    LocationManager,
    Memory,
    Modem,
    Monitor,
    Mouse,
    MultipleUsers,
    Numbers,
    QuicktimeSettings,
    RemoteAccess,
    SoftwareUpdate,
    Sound,
    Speech,
    StartupDisk,
    Tcpip,
    Text,
    WebSharing,
}

impl ControlPanelIcon {
    pub fn filename(&self) -> &str {
        match self {
            ControlPanelIcon::AdobeGamma => "Adobe gamma.png",
            ControlPanelIcon::Appearance => "Appearance.png",
            ControlPanelIcon::AppleMenuOptions => "Apple menu options.png",
            ControlPanelIcon::Appletalk => "Appletalk.png",
            ControlPanelIcon::Atm => "Atm.png",
            ControlPanelIcon::Colorsync => "Colorsync.png",
            ControlPanelIcon::ControlStrip => "Control strip.png",
            ControlPanelIcon::DateAndTime => "Date and time.png",
            ControlPanelIcon::DialAssist => "Dial assist.png",
            ControlPanelIcon::EnergySaver => "Energy saver.png",
            ControlPanelIcon::ExtensionsManager => "Extensions manager.png",
            ControlPanelIcon::FileExchange => "File exchange.png",
            ControlPanelIcon::FileSharing => "File sharing.png",
            ControlPanelIcon::GeneralControls => "General controls.png",
            ControlPanelIcon::Internet => "Internet.png",
            ControlPanelIcon::Keyboard => "Keyboard.png",
            ControlPanelIcon::KeychainAccess => "Keychain access.png",
            ControlPanelIcon::Launcher => "Launcher.png",
            ControlPanelIcon::LocationManager => "Location manager.png",
            ControlPanelIcon::Memory => "Memory.png",
            ControlPanelIcon::Modem => "Modem.png",
            ControlPanelIcon::Monitor => "Monitor.png",
            ControlPanelIcon::Mouse => "Mouse.png",
            ControlPanelIcon::MultipleUsers => "Multiple users.png",
            ControlPanelIcon::Numbers => "Numbers.png",
            ControlPanelIcon::QuicktimeSettings => "Quicktime settings.png",
            ControlPanelIcon::RemoteAccess => "Remote access.png",
            ControlPanelIcon::SoftwareUpdate => "Software update.png",
            ControlPanelIcon::Sound => "Sound.png",
            ControlPanelIcon::Speech => "Speech.png",
            ControlPanelIcon::StartupDisk => "Startup disk.png",
            ControlPanelIcon::Tcpip => "TCPIP.png",
            ControlPanelIcon::Text => "Text.png",
            ControlPanelIcon::WebSharing => "Web sharing.png",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ControlPanelIcon::AdobeGamma => "Adobe Gamma",
            ControlPanelIcon::Appearance => "Appearance",
            ControlPanelIcon::AppleMenuOptions => "Apple Menu Options",
            ControlPanelIcon::Appletalk => "AppleTalk",
            ControlPanelIcon::Atm => "ATM",
            ControlPanelIcon::Colorsync => "ColorSync",
            ControlPanelIcon::ControlStrip => "Control Strip",
            ControlPanelIcon::DateAndTime => "Date and Time",
            ControlPanelIcon::DialAssist => "Dial Assist",
            ControlPanelIcon::EnergySaver => "Energy Saver",
            ControlPanelIcon::ExtensionsManager => "Extensions Manager",
            ControlPanelIcon::FileExchange => "File Exchange",
            ControlPanelIcon::FileSharing => "File Sharing",
            ControlPanelIcon::GeneralControls => "General Controls",
            ControlPanelIcon::Internet => "Internet",
            ControlPanelIcon::Keyboard => "Keyboard",
            ControlPanelIcon::KeychainAccess => "Keychain Access",
            ControlPanelIcon::Launcher => "Launcher",
            ControlPanelIcon::LocationManager => "Location Manager",
            ControlPanelIcon::Memory => "Memory",
            ControlPanelIcon::Modem => "Modem",
            ControlPanelIcon::Monitor => "Monitor",
            ControlPanelIcon::Mouse => "Mouse",
            ControlPanelIcon::MultipleUsers => "Multiple Users",
            ControlPanelIcon::Numbers => "Numbers",
            ControlPanelIcon::QuicktimeSettings => "QuickTime Settings",
            ControlPanelIcon::RemoteAccess => "Remote Access",
            ControlPanelIcon::SoftwareUpdate => "Software Update",
            ControlPanelIcon::Sound => "Sound",
            ControlPanelIcon::Speech => "Speech",
            ControlPanelIcon::StartupDisk => "Startup Disk",
            ControlPanelIcon::Tcpip => "TCP/IP",
            ControlPanelIcon::Text => "Text",
            ControlPanelIcon::WebSharing => "Web Sharing",
        }
    }

    pub const ALL: [ControlPanelIcon; 34] = [
        ControlPanelIcon::AdobeGamma,
        ControlPanelIcon::Appearance,
        ControlPanelIcon::AppleMenuOptions,
        ControlPanelIcon::Appletalk,
        ControlPanelIcon::Atm,
        ControlPanelIcon::Colorsync,
        ControlPanelIcon::ControlStrip,
        ControlPanelIcon::DateAndTime,
        ControlPanelIcon::DialAssist,
        ControlPanelIcon::EnergySaver,
        ControlPanelIcon::ExtensionsManager,
        ControlPanelIcon::FileExchange,
        ControlPanelIcon::FileSharing,
        ControlPanelIcon::GeneralControls,
        ControlPanelIcon::Internet,
        ControlPanelIcon::Keyboard,
        ControlPanelIcon::KeychainAccess,
        ControlPanelIcon::Launcher,
        ControlPanelIcon::LocationManager,
        ControlPanelIcon::Memory,
        ControlPanelIcon::Modem,
        ControlPanelIcon::Monitor,
        ControlPanelIcon::Mouse,
        ControlPanelIcon::MultipleUsers,
        ControlPanelIcon::Numbers,
        ControlPanelIcon::QuicktimeSettings,
        ControlPanelIcon::RemoteAccess,
        ControlPanelIcon::SoftwareUpdate,
        ControlPanelIcon::Sound,
        ControlPanelIcon::Speech,
        ControlPanelIcon::StartupDisk,
        ControlPanelIcon::Tcpip,
        ControlPanelIcon::Text,
        ControlPanelIcon::WebSharing,
    ];
}

/// Control Strip icons (Sound Volume, Monitor, CD, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ControlStripIcon {
    AppleLocation,
    AppleTalk,
    Cd,
    FileSharing,
    Itunes,
    KeychainStrip,
    MonitorBitdepth,
    MonitorResolution,
    Printer,
    RemoteAccess,
    SoundVolume,
    WebSharing,
}

impl ControlStripIcon {
    pub fn filename(&self) -> &str {
        match self {
            ControlStripIcon::AppleLocation => "Apple location.png",
            ControlStripIcon::AppleTalk => "Apple talk.png",
            ControlStripIcon::Cd => "CD.png",
            ControlStripIcon::FileSharing => "File sharing.png",
            ControlStripIcon::Itunes => "iTunes.png",
            ControlStripIcon::KeychainStrip => "Keychain strip.png",
            ControlStripIcon::MonitorBitdepth => "Monitor bitdepth.png",
            ControlStripIcon::MonitorResolution => "Monitor resolution.png",
            ControlStripIcon::Printer => "Printer.png",
            ControlStripIcon::RemoteAccess => "Remote access.png",
            ControlStripIcon::SoundVolume => "Sound volume.png",
            ControlStripIcon::WebSharing => "Web sharing.png",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ControlStripIcon::AppleLocation => "Apple Location",
            ControlStripIcon::AppleTalk => "Apple Talk",
            ControlStripIcon::Cd => "CD",
            ControlStripIcon::FileSharing => "File Sharing",
            ControlStripIcon::Itunes => "iTunes",
            ControlStripIcon::KeychainStrip => "Keychain Strip",
            ControlStripIcon::MonitorBitdepth => "Monitor Bit Depth",
            ControlStripIcon::MonitorResolution => "Monitor Resolution",
            ControlStripIcon::Printer => "Printer",
            ControlStripIcon::RemoteAccess => "Remote Access",
            ControlStripIcon::SoundVolume => "Sound Volume",
            ControlStripIcon::WebSharing => "Web Sharing",
        }
    }

    pub const ALL: [ControlStripIcon; 12] = [
        ControlStripIcon::AppleLocation,
        ControlStripIcon::AppleTalk,
        ControlStripIcon::Cd,
        ControlStripIcon::FileSharing,
        ControlStripIcon::Itunes,
        ControlStripIcon::KeychainStrip,
        ControlStripIcon::MonitorBitdepth,
        ControlStripIcon::MonitorResolution,
        ControlStripIcon::Printer,
        ControlStripIcon::RemoteAccess,
        ControlStripIcon::SoundVolume,
        ControlStripIcon::WebSharing,
    ];
}

/// Folder icons (Applications, Extensions, Fonts, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FolderIcon {
    AppleMenuItem,
    ApplicationSupport,
    Applications,
    Assistant,
    ColorSyncprofiles,
    ContextualMenuItems,
    ControlPanels,
    ControlStrip,
    Default,
    Extensions,
    Extras,
    Favorites,
    Fonts,
    Help,
    InternetSearchSites,
    Internet,
    LanguageAndRegion,
    Preferences,
    RecentDocuments,
    Scripts,
    StartupItems,
    System,
    TextEncodings,
    Utilities,
}

impl FolderIcon {
    pub fn filename(&self) -> &str {
        match self {
            FolderIcon::AppleMenuItem => "Apple menu item.png",
            FolderIcon::ApplicationSupport => "Application Support.png",
            FolderIcon::Applications => "Applications.png",
            FolderIcon::Assistant => "Assistant.png",
            FolderIcon::ColorSyncprofiles => "Colorsync profiles.png",
            FolderIcon::ContextualMenuItems => "Contextual menu items.png",
            FolderIcon::ControlPanels => "Control panels.png",
            FolderIcon::ControlStrip => "Control strip.png",
            FolderIcon::Default => "Default.png",
            FolderIcon::Extensions => "Extensions.png",
            FolderIcon::Extras => "Extras.png",
            FolderIcon::Favorites => "Favorites.png",
            FolderIcon::Fonts => "Fonts.png",
            FolderIcon::Help => "Help.png",
            FolderIcon::InternetSearchSites => "Internet search sites.png",
            FolderIcon::Internet => "Internet.png",
            FolderIcon::LanguageAndRegion => "Language and region.png",
            FolderIcon::Preferences => "Preferences.png",
            FolderIcon::RecentDocuments => "Recent documents.png",
            FolderIcon::Scripts => "Scripts.png",
            FolderIcon::StartupItems => "Startup items.png",
            FolderIcon::System => "System.png",
            FolderIcon::TextEncodings => "Text encodings.png",
            FolderIcon::Utilities => "Utilities.png",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            FolderIcon::AppleMenuItem => "Apple Menu Item",
            FolderIcon::ApplicationSupport => "Application Support",
            FolderIcon::Applications => "Applications",
            FolderIcon::Assistant => "Assistant",
            FolderIcon::ColorSyncprofiles => "ColorSync Profiles",
            FolderIcon::ContextualMenuItems => "Contextual Menu Items",
            FolderIcon::ControlPanels => "Control Panels",
            FolderIcon::ControlStrip => "Control Strip",
            FolderIcon::Default => "Default",
            FolderIcon::Extensions => "Extensions",
            FolderIcon::Extras => "Extras",
            FolderIcon::Favorites => "Favorites",
            FolderIcon::Fonts => "Fonts",
            FolderIcon::Help => "Help",
            FolderIcon::InternetSearchSites => "Internet Search Sites",
            FolderIcon::Internet => "Internet",
            FolderIcon::LanguageAndRegion => "Language and Region",
            FolderIcon::Preferences => "Preferences",
            FolderIcon::RecentDocuments => "Recent Documents",
            FolderIcon::Scripts => "Scripts",
            FolderIcon::StartupItems => "Startup Items",
            FolderIcon::System => "System",
            FolderIcon::TextEncodings => "Text Encodings",
            FolderIcon::Utilities => "Utilities",
        }
    }

    pub const ALL: [FolderIcon; 24] = [
        FolderIcon::AppleMenuItem,
        FolderIcon::ApplicationSupport,
        FolderIcon::Applications,
        FolderIcon::Assistant,
        FolderIcon::ColorSyncprofiles,
        FolderIcon::ContextualMenuItems,
        FolderIcon::ControlPanels,
        FolderIcon::ControlStrip,
        FolderIcon::Default,
        FolderIcon::Extensions,
        FolderIcon::Extras,
        FolderIcon::Favorites,
        FolderIcon::Fonts,
        FolderIcon::Help,
        FolderIcon::InternetSearchSites,
        FolderIcon::Internet,
        FolderIcon::LanguageAndRegion,
        FolderIcon::Preferences,
        FolderIcon::RecentDocuments,
        FolderIcon::Scripts,
        FolderIcon::StartupItems,
        FolderIcon::System,
        FolderIcon::TextEncodings,
        FolderIcon::Utilities,
    ];
}

/// Menu Bar icons (Finder, Control Panels, Favorites, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuBarIcon {
    AppleLogo,
    AppleSystem,
    Calculator,
    Chooser,
    ControlPanels,
    Favorites,
    Finder,
    KeyCaps,
    NetworkBrowser,
    RecentApplications,
    RecentDocuments,
    RemoteAccess,
    Scrapbook,
    Sherlock20,
    Stickies,
    Suitcase,
}

impl MenuBarIcon {
    pub fn filename(&self) -> &str {
        match self {
            MenuBarIcon::AppleLogo => "Apple logo.png",
            MenuBarIcon::AppleSystem => "Apple system.png",
            MenuBarIcon::Calculator => "Calculator.png",
            MenuBarIcon::Chooser => "Chooser.png",
            MenuBarIcon::ControlPanels => "Control panels.png",
            MenuBarIcon::Favorites => "Favorites.png",
            MenuBarIcon::Finder => "Finder.png",
            MenuBarIcon::KeyCaps => "Key caps.png",
            MenuBarIcon::NetworkBrowser => "Network browser.png",
            MenuBarIcon::RecentApplications => "Recent applications.png",
            MenuBarIcon::RecentDocuments => "Recent documents.png",
            MenuBarIcon::RemoteAccess => "Remote access.png",
            MenuBarIcon::Scrapbook => "Scrapbook.png",
            MenuBarIcon::Sherlock20 => "Sherlock 2.0.png",
            MenuBarIcon::Stickies => "Stickies.png",
            MenuBarIcon::Suitcase => "Suitcase.png",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            MenuBarIcon::AppleLogo => "Apple Logo",
            MenuBarIcon::AppleSystem => "Apple System",
            MenuBarIcon::Calculator => "Calculator",
            MenuBarIcon::Chooser => "Chooser",
            MenuBarIcon::ControlPanels => "Control Panels",
            MenuBarIcon::Favorites => "Favorites",
            MenuBarIcon::Finder => "Finder",
            MenuBarIcon::KeyCaps => "Key Caps",
            MenuBarIcon::NetworkBrowser => "Network Browser",
            MenuBarIcon::RecentApplications => "Recent Applications",
            MenuBarIcon::RecentDocuments => "Recent Documents",
            MenuBarIcon::RemoteAccess => "Remote Access",
            MenuBarIcon::Scrapbook => "Scrapbook",
            MenuBarIcon::Sherlock20 => "Sherlock 2.0",
            MenuBarIcon::Stickies => "Stickies",
            MenuBarIcon::Suitcase => "Suitcase",
        }
    }

    pub const ALL: [MenuBarIcon; 16] = [
        MenuBarIcon::AppleLogo,
        MenuBarIcon::AppleSystem,
        MenuBarIcon::Calculator,
        MenuBarIcon::Chooser,
        MenuBarIcon::ControlPanels,
        MenuBarIcon::Favorites,
        MenuBarIcon::Finder,
        MenuBarIcon::KeyCaps,
        MenuBarIcon::NetworkBrowser,
        MenuBarIcon::RecentApplications,
        MenuBarIcon::RecentDocuments,
        MenuBarIcon::RemoteAccess,
        MenuBarIcon::Scrapbook,
        MenuBarIcon::Sherlock20,
        MenuBarIcon::Stickies,
        MenuBarIcon::Suitcase,
    ];
}

/// Top-level glyph enum for classic Mac OS icons.
///
/// Supports all 6 icon categories:
/// - System (41 icons)
/// - Applications (22 icons)
/// - ControlPanel (34 icons)
/// - ControlStrip (12 icons)
/// - Folder (24 icons)
/// - MenuBar (16 icons)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconClassicGlyph {
    System(SystemIcon),
    Applications(ApplicationsIcon),
    ControlPanel(ControlPanelIcon),
    ControlStrip(ControlStripIcon),
    Folder(FolderIcon),
    MenuBar(MenuBarIcon),
}

impl IconClassicGlyph {
    /// Returns the category directory name (e.g., "system", "applications").
    pub fn category(&self) -> &str {
        match self {
            IconClassicGlyph::System(_) => "system",
            IconClassicGlyph::Applications(_) => "applications",
            IconClassicGlyph::ControlPanel(_) => "control-panel",
            IconClassicGlyph::ControlStrip(_) => "control-strip",
            IconClassicGlyph::Folder(_) => "folder",
            IconClassicGlyph::MenuBar(_) => "menu-bar",
        }
    }

    /// Returns the icon filename (with extension) for this glyph.
    pub fn filename(&self) -> &str {
        match self {
            IconClassicGlyph::System(icon) => icon.filename(),
            IconClassicGlyph::Applications(icon) => icon.filename(),
            IconClassicGlyph::ControlPanel(icon) => icon.filename(),
            IconClassicGlyph::ControlStrip(icon) => icon.filename(),
            IconClassicGlyph::Folder(icon) => icon.filename(),
            IconClassicGlyph::MenuBar(icon) => icon.filename(),
        }
    }

    /// Returns a human-readable label for this glyph.
    pub fn label(&self) -> &str {
        match self {
            IconClassicGlyph::System(icon) => icon.label(),
            IconClassicGlyph::Applications(icon) => icon.label(),
            IconClassicGlyph::ControlPanel(icon) => icon.label(),
            IconClassicGlyph::ControlStrip(icon) => icon.label(),
            IconClassicGlyph::Folder(icon) => icon.label(),
            IconClassicGlyph::MenuBar(icon) => icon.label(),
        }
    }

    /// Returns the icon identifier for embedding/lookup (e.g., "system_hard_drive").
    pub fn ident(&self) -> String {
        let category = self.category();
        let filename = self.filename();
        let name_without_ext = filename.strip_suffix(".png").unwrap_or(filename);
        format!(
            "{}_{}",
            category,
            name_without_ext
                .to_lowercase()
                .replace(" ", "_")
                .replace("(", "")
                .replace(")", "")
                .replace(".", "")
        )
    }
}

/// A classic Mac OS icon element.
///
/// Renders as an `<img>` element with the icon source and alt text.
/// No sizing, styling, or visibility controls — use CSS for layout.
///
/// # Example
///
/// ```ignore
/// use iti::components::icon_classic::{IconClassic, IconClassicGlyph, SystemIcon};
/// use mogwai::web::prelude::*;
///
/// let icon = IconClassic::<Web>::new(
///     IconClassicGlyph::System(SystemIcon::HardDrive)
/// );
/// mogwai::web::body().append_child(&icon);
/// ```
#[derive(ViewChild)]
pub struct IconClassic<V: View> {
    #[child]
    img: V::Element,
}

impl<V: View> IconClassic<V> {
    /// Create a new classic icon from a glyph.
    ///
    /// With the `embed-assets` feature, icons are compiled into the WASM binary
    /// and served via Blob URLs. Without it, icons load from
    /// `/icons-classic/{category}/{filename}.png`.
    pub fn new(glyph: IconClassicGlyph) -> Self {
        let src = if cfg!(feature = "embed-assets") {
            #[cfg(feature = "embed-assets")]
            {
                crate::assets::embedded::blob_url_for_classic_icon(&glyph)
            }
            #[cfg(not(feature = "embed-assets"))]
            {
                unreachable!()
            }
        } else {
            format!("/icons-classic/{}/{}", glyph.category(), glyph.filename())
        };

        rsx! {
            let img = img(src = src, alt = glyph.label()) {}
        }

        Self { img }
    }
}
