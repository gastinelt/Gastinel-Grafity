//! Windows-like First-Boot Setup Wizard (OOBE) for Gastinel Alpine Linux.
//!
//! Provides a modern, elegant, fully-graphical setup experience on first boot:
//! 1. Welcome & Keyboard layout (AZERTY / QWERTY)
//! 2. Wi-Fi / Network setup (with offline skip option)
//! 3. User account & password creation
//! 4. Disk & Partition target selection
//! 5. Installation progress & system optimization
//! 6. Completion & launch into Gastinel Desktop

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect, Size};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WizardStep {
    WelcomeLanguage = 0,
    Network = 1,
    UserAccount = 2,
    DiskTarget = 3,
    Installing = 4,
    Complete = 5,
}

#[derive(Clone, Debug)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_percent: u8,
    pub is_secure: bool,
}

pub struct SetupWizard {
    pub current_step: WizardStep,
    pub screen_size: Size,
    pub card_bounds: Rect,

    // Step 1: Language / Keyboard
    pub selected_keyboard: usize, // 0: Français (AZERTY), 1: US (QWERTY), 2: Suisse (QWERTZ)
    pub key_test_input: String,

    // Step 2: Wi-Fi
    pub wifi_networks: Vec<WifiNetwork>,
    pub selected_wifi: Option<usize>,
    pub wifi_password: String,

    // Step 3: User Account
    pub full_name: String,
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub hostname: String,
    pub active_field: usize, // 0: name, 1: username, 2: password, 3: confirm, 4: hostname

    // Step 4: Disk
    pub selected_disk_mode: usize, // 0: Auto Full Disk, 1: Dual-boot partition, 2: USB Nomade

    // Step 5: Install progress
    pub install_progress: f32, // 0.0..1.0
    pub install_status_text: String,

    pub is_finished: bool,
}

impl SetupWizard {
    pub const CARD_WIDTH: u32 = 820;
    pub const CARD_HEIGHT: u32 = 560;

    pub fn new(screen_size: Size) -> Self {
        let card_x = (screen_size.width.saturating_sub(Self::CARD_WIDTH) / 2) as i32;
        let card_y = (screen_size.height.saturating_sub(Self::CARD_HEIGHT) / 2) as i32;

        let mut wizard = Self {
            current_step: WizardStep::WelcomeLanguage,
            screen_size,
            card_bounds: Rect::new(card_x, card_y, Self::CARD_WIDTH, Self::CARD_HEIGHT),

            selected_keyboard: 0,
            key_test_input: String::new(),

            wifi_networks: Vec::new(),
            selected_wifi: None,
            wifi_password: String::new(),

            full_name: String::from("Théo"),
            username: String::from("gastinel"),
            password: String::new(),
            confirm_password: String::new(),
            hostname: String::from("gastinel-book5"),
            active_field: 0,

            selected_disk_mode: 0,

            install_progress: 0.0,
            install_status_text: String::from("Préparation du système..."),

            is_finished: false,
        };

        // Populate sample Wi-Fi networks
        wizard.wifi_networks.push(WifiNetwork {
            ssid: String::from("Wi-Fi 6 Ultra (5 GHz)"),
            signal_percent: 94,
            is_secure: true,
        });
        wizard.wifi_networks.push(WifiNetwork {
            ssid: String::from("Livebox-Gastinel"),
            signal_percent: 86,
            is_secure: true,
        });
        wizard.wifi_networks.push(WifiNetwork {
            ssid: String::from("Freebox-Connexion"),
            signal_percent: 65,
            is_secure: true,
        });

        wizard
    }

    pub fn resize(&mut self, screen_size: Size) {
        self.screen_size = screen_size;
        self.card_bounds.x = (screen_size.width.saturating_sub(Self::CARD_WIDTH) / 2) as i32;
        self.card_bounds.y = (screen_size.height.saturating_sub(Self::CARD_HEIGHT) / 2) as i32;
    }

    pub fn next_step(&mut self) {
        match self.current_step {
            WizardStep::WelcomeLanguage => self.current_step = WizardStep::Network,
            WizardStep::Network => self.current_step = WizardStep::UserAccount,
            WizardStep::UserAccount => self.current_step = WizardStep::DiskTarget,
            WizardStep::DiskTarget => {
                self.current_step = WizardStep::Installing;
                self.install_progress = 0.15;
                self.install_status_text = String::from("Déballage du système de base Alpine Linux...");
            }
            WizardStep::Installing => {
                self.current_step = WizardStep::Complete;
            }
            WizardStep::Complete => {
                self.is_finished = true;
            }
        }
    }

    pub fn prev_step(&mut self) {
        match self.current_step {
            WizardStep::WelcomeLanguage => {}
            WizardStep::Network => self.current_step = WizardStep::WelcomeLanguage,
            WizardStep::UserAccount => self.current_step = WizardStep::Network,
            WizardStep::DiskTarget => self.current_step = WizardStep::UserAccount,
            WizardStep::Installing => {}
            WizardStep::Complete => {}
        }
    }

    pub fn tick_installation(&mut self, delta: f32) {
        if self.current_step == WizardStep::Installing {
            self.install_progress = (self.install_progress + delta).min(1.0);
            if self.install_progress < 0.35 {
                self.install_status_text = String::from("Déballage du système de base Alpine Linux (musl libc)...");
            } else if self.install_progress < 0.65 {
                self.install_status_text = String::from("Configuration d'OpenRC, de seatd et des permissions...");
            } else if self.install_progress < 0.85 {
                self.install_status_text = String::from("Application des optimisations mémoire (zram, swappiness 10)...");
            } else if self.install_progress < 0.99 {
                self.install_status_text = String::from("Finalisation du chargeur EFI et de Gastinel Desktop...");
            } else {
                self.next_step();
            }
        }
    }

    /// Renders the setup wizard into the RenderQueue using procedural SDF quads and Inter typography.
    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // 1. Dark ambient background
        let bg = QuadInstance::solid(
            Rect::new(0, 0, self.screen_size.width, self.screen_size.height),
            [0.05, 0.06, 0.08, 1.0],
            0.05,
        );
        queue.push(bg);

        // 2. Central Setup Card (Acrylic rounded box with deep soft drop shadow)
        let card = QuadInstance::rounded_box(
            self.card_bounds,
            16.0,
            [0.10, 0.11, 0.14, 0.96], // Dark acrylic card
            [0.24, 0.26, 0.32, 0.80], // Refined border
            1.5,
            0.5,
        )
        .with_shadow(
            [0.0, 0.0, 0.0, 0.60],
            32.0,
            [0.0, 12.0],
        );
        queue.push(card);

        // 3. Top Banner: Gastinel Logo & Step Indicators
        let banner_rect = Rect::new(
            self.card_bounds.x,
            self.card_bounds.y,
            self.card_bounds.width,
            64,
        );
        queue.push(QuadInstance::rounded_box(
            banner_rect,
            16.0,
            [0.13, 0.14, 0.18, 0.90],
            [0.20, 0.22, 0.26, 0.40],
            1.0,
            0.52,
        ));

        atlas.layout_text(
            "GASTINEL ALPINE",
            Point::new(self.card_bounds.x + 28, self.card_bounds.y + 40),
            15,
            FontWeight::Bold,
            [0.25, 0.55, 0.98, 1.0], // Electric blue branding
            glyphs,
        );

        atlas.layout_text(
            "Assistant de Configuration Rapide",
            Point::new(self.card_bounds.x + 190, self.card_bounds.y + 40),
            13,
            FontWeight::Regular,
            [0.60, 0.64, 0.72, 1.0],
            glyphs,
        );

        // Step breadcrumb dots
        let dots_start_x = self.card_bounds.right() - 160;
        for i in 0..6 {
            let dot_x = dots_start_x + i * 22;
            let is_current = i == (self.current_step as i32);
            let is_past = i < (self.current_step as i32);

            let dot_color = if is_current {
                [0.25, 0.55, 0.98, 1.0] // active
            } else if is_past {
                [0.45, 0.80, 0.55, 1.0] // completed green
            } else {
                [0.22, 0.24, 0.28, 0.6] // upcoming
            };

            queue.push(QuadInstance::rounded_box(
                Rect::new(dot_x, self.card_bounds.y + 28, 10, 10),
                5.0,
                dot_color,
                [0.0, 0.0, 0.0, 0.0],
                0.0,
                0.53,
            ));
        }

        // 4. Render Step Content
        let content_x = self.card_bounds.x + 40;
        let content_y = self.card_bounds.y + 95;

        match self.current_step {
            WizardStep::WelcomeLanguage => {
                self.render_welcome(content_x, content_y, atlas, queue, glyphs);
            }
            WizardStep::Network => {
                self.render_network(content_x, content_y, atlas, queue, glyphs);
            }
            WizardStep::UserAccount => {
                self.render_user_account(content_x, content_y, atlas, queue, glyphs);
            }
            WizardStep::DiskTarget => {
                self.render_disk_target(content_x, content_y, atlas, queue, glyphs);
            }
            WizardStep::Installing => {
                self.render_installing(content_x, content_y, atlas, queue, glyphs);
            }
            WizardStep::Complete => {
                self.render_complete(content_x, content_y, atlas, queue, glyphs);
            }
        }

        // 5. Navigation Bar (Bottom)
        if self.current_step != WizardStep::Installing {
            self.render_navigation_buttons(atlas, queue, glyphs);
        }
    }

    fn render_welcome(
        &self,
        x: i32,
        y: i32,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        atlas.layout_text(
            "Bienvenue dans votre nouvel environnement",
            Point::new(x, y + 26),
            20,
            FontWeight::Bold,
            [0.96, 0.97, 1.0, 1.0],
            glyphs,
        );

        atlas.layout_text(
            "Choisissez votre langue et la disposition de votre clavier pour commencer :",
            Point::new(x, y + 54),
            13,
            FontWeight::Regular,
            [0.65, 0.68, 0.75, 1.0],
            glyphs,
        );

        // Keyboard option cards
        let options = [
            ("Français (AZERTY)", "Disposition standard pour la France & Belgique"),
            ("English (US QWERTY)", "Standard international QWERTY layout"),
            ("Suisse (Français / Deutsch)", "Disposition bilingue Suisse"),
        ];

        for (i, (title, desc)) in options.iter().enumerate() {
            let opt_y = y + 80 + (i as i32) * 58;
            let opt_rect = Rect::new(x, opt_y, self.card_bounds.width - 80, 48);

            let is_sel = i == self.selected_keyboard;
            let (fill, border) = if is_sel {
                (
                    [0.22, 0.45, 0.90, 0.85],
                    [0.40, 0.65, 1.00, 0.95],
                )
            } else {
                (
                    [0.14, 0.15, 0.18, 0.60],
                    [0.24, 0.26, 0.30, 0.50],
                )
            };

            queue.push(QuadInstance::rounded_box(
                opt_rect,
                8.0,
                fill,
                border,
                1.0,
                0.55,
            ));

            atlas.layout_text(
                title,
                Point::new(x + 20, opt_y + 22),
                14,
                FontWeight::SemiBold,
                if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.90, 0.92, 0.96, 1.0] },
                glyphs,
            );

            atlas.layout_text(
                desc,
                Point::new(x + 20, opt_y + 38),
                12,
                FontWeight::Regular,
                if is_sel { [0.85, 0.90, 1.0, 0.9] } else { [0.55, 0.58, 0.65, 1.0] },
                glyphs,
            );
        }

        // Test field
        let test_y = y + 270;
        atlas.layout_text(
            "Zone de test du clavier :",
            Point::new(x, test_y + 16),
            12,
            FontWeight::SemiBold,
            [0.70, 0.74, 0.80, 1.0],
            glyphs,
        );

        let test_rect = Rect::new(x, test_y + 24, self.card_bounds.width - 80, 40);
        queue.push(QuadInstance::rounded_box(
            test_rect,
            6.0,
            [0.12, 0.13, 0.16, 0.80],
            [0.28, 0.30, 0.36, 0.60],
            1.0,
            0.55,
        ));

        let display_test = if self.key_test_input.is_empty() {
            "Tapez quelques caractères ici pour vérifier les accents (é, è, à, ç, @, €)..."
        } else {
            &self.key_test_input
        };

        atlas.layout_text(
            display_test,
            Point::new(x + 16, test_y + 48),
            13,
            FontWeight::Regular,
            if self.key_test_input.is_empty() {
                [0.45, 0.48, 0.54, 1.0]
            } else {
                [0.98, 0.98, 1.0, 1.0]
            },
            glyphs,
        );
    }

    fn render_network(
        &self,
        x: i32,
        y: i32,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        atlas.layout_text(
            "Connexion Réseau & Wi-Fi",
            Point::new(x, y + 26),
            20,
            FontWeight::Bold,
            [0.96, 0.97, 1.0, 1.0],
            glyphs,
        );

        atlas.layout_text(
            "Connectez-vous pour activer les miroirs de paquets officiels Alpine :",
            Point::new(x, y + 54),
            13,
            FontWeight::Regular,
            [0.65, 0.68, 0.75, 1.0],
            glyphs,
        );

        for (i, net) in self.wifi_networks.iter().enumerate() {
            let item_y = y + 80 + (i as i32) * 54;
            let item_rect = Rect::new(x, item_y, self.card_bounds.width - 80, 46);

            let is_sel = self.selected_wifi == Some(i);
            let fill = if is_sel {
                [0.20, 0.40, 0.85, 0.70]
            } else {
                [0.14, 0.15, 0.18, 0.60]
            };

            queue.push(QuadInstance::rounded_box(
                item_rect,
                8.0,
                fill,
                [0.26, 0.28, 0.34, 0.60],
                1.0,
                0.55,
            ));

            // Network name
            atlas.layout_text(
                &net.ssid,
                Point::new(x + 18, item_y + 28),
                14,
                FontWeight::SemiBold,
                [0.95, 0.96, 0.98, 1.0],
                glyphs,
            );

            // Signal strength pill
            let sig_text = format!("Signal: {}%", net.signal_percent);
            atlas.layout_text(
                &sig_text,
                Point::new(item_rect.right() - 110, item_y + 28),
                12,
                FontWeight::Medium,
                [0.45, 0.85, 0.55, 1.0],
                glyphs,
            );
        }

        // Password box
        let pwd_y = y + 260;
        atlas.layout_text(
            "Clé de sécurité Wi-Fi (WPA2/WPA3) :",
            Point::new(x, pwd_y + 16),
            12,
            FontWeight::SemiBold,
            [0.70, 0.74, 0.80, 1.0],
            glyphs,
        );

        let pwd_rect = Rect::new(x, pwd_y + 24, self.card_bounds.width - 80, 40);
        queue.push(QuadInstance::rounded_box(
            pwd_rect,
            6.0,
            [0.12, 0.13, 0.16, 0.80],
            [0.28, 0.30, 0.36, 0.60],
            1.0,
            0.55,
        ));

        let pwd_display = if self.wifi_password.is_empty() {
            "Entrez le mot de passe réseau..."
        } else {
            "••••••••••••"
        };

        atlas.layout_text(
            pwd_display,
            Point::new(x + 16, pwd_y + 48),
            13,
            FontWeight::Regular,
            [0.55, 0.58, 0.65, 1.0],
            glyphs,
        );
    }

    fn render_user_account(
        &self,
        x: i32,
        y: i32,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        atlas.layout_text(
            "Création de votre Compte",
            Point::new(x, y + 26),
            20,
            FontWeight::Bold,
            [0.96, 0.97, 1.0, 1.0],
            glyphs,
        );

        atlas.layout_text(
            "Configurez votre utilisateur principal pour votre session Gastinel :",
            Point::new(x, y + 54),
            13,
            FontWeight::Regular,
            [0.65, 0.68, 0.75, 1.0],
            glyphs,
        );

        // Fields: Nom complet, Utilisateur, Hostname
        let fields: [(&str, &str); 4] = [
            ("Votre nom", self.full_name.as_str()),
            ("Nom d'utilisateur (login)", self.username.as_str()),
            ("Nom de l'ordinateur (hostname)", self.hostname.as_str()),
            ("Mot de passe", "••••••••"),
        ];

        for (i, (label, val)) in fields.iter().enumerate() {
            let fy = y + 80 + (i as i32) * 58;
            atlas.layout_text(
                label,
                Point::new(x, fy + 14),
                12,
                FontWeight::SemiBold,
                [0.70, 0.73, 0.80, 1.0],
                glyphs,
            );

            let f_rect = Rect::new(x, fy + 20, self.card_bounds.width - 80, 34);
            queue.push(QuadInstance::rounded_box(
                f_rect,
                6.0,
                [0.13, 0.14, 0.17, 0.80],
                [0.26, 0.28, 0.34, 0.60],
                1.0,
                0.55,
            ));

            atlas.layout_text(
                val,
                Point::new(x + 14, fy + 42),
                13,
                FontWeight::Medium,
                [0.95, 0.96, 0.98, 1.0],
                glyphs,
            );
        }
    }

    fn render_disk_target(
        &self,
        x: i32,
        y: i32,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        atlas.layout_text(
            "Destination de l'Installation",
            Point::new(x, y + 26),
            20,
            FontWeight::Bold,
            [0.96, 0.97, 1.0, 1.0],
            glyphs,
        );

        atlas.layout_text(
            "Sélectionnez le mode de déploiement natif sur votre machine :",
            Point::new(x, y + 54),
            13,
            FontWeight::Regular,
            [0.65, 0.68, 0.75, 1.0],
            glyphs,
        );

        let disks = [
            ("Installation Automatique Dédiée (Recommandé)", "Utilise le SSD principal, optimise les partitions et active la compression zram"),
            ("Dual-Boot avec Windows (Partition existante)", "Préserve votre installation Windows actuelle et installe Gastinel à côté"),
            ("Installation Nomade sur Clé USB / SSD Externe", "Crée un système complet bootable ultra-léger et portable"),
        ];

        for (i, (title, desc)) in disks.iter().enumerate() {
            let dy = y + 80 + (i as i32) * 64;
            let d_rect = Rect::new(x, dy, self.card_bounds.width - 80, 54);

            let is_sel = i == self.selected_disk_mode;
            let (fill, border) = if is_sel {
                (
                    [0.22, 0.45, 0.90, 0.85],
                    [0.40, 0.65, 1.00, 0.95],
                )
            } else {
                (
                    [0.14, 0.15, 0.18, 0.60],
                    [0.24, 0.26, 0.30, 0.50],
                )
            };

            queue.push(QuadInstance::rounded_box(
                d_rect,
                8.0,
                fill,
                border,
                1.0,
                0.55,
            ));

            atlas.layout_text(
                title,
                Point::new(x + 20, dy + 24),
                14,
                FontWeight::SemiBold,
                if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.90, 0.92, 0.96, 1.0] },
                glyphs,
            );

            atlas.layout_text(
                desc,
                Point::new(x + 20, dy + 42),
                12,
                FontWeight::Regular,
                if is_sel { [0.85, 0.90, 1.0, 0.9] } else { [0.55, 0.58, 0.65, 1.0] },
                glyphs,
            );
        }
    }

    fn render_installing(
        &self,
        x: i32,
        y: i32,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        atlas.layout_text(
            "Installation & Optimisations en cours...",
            Point::new(x, y + 40),
            20,
            FontWeight::Bold,
            [0.96, 0.97, 1.0, 1.0],
            glyphs,
        );

        atlas.layout_text(
            &self.install_status_text,
            Point::new(x, y + 80),
            14,
            FontWeight::Medium,
            [0.75, 0.80, 0.90, 1.0],
            glyphs,
        );

        // Progress bar container
        let bar_w = self.card_bounds.width - 80;
        let bar_h = 16;
        let bar_rect = Rect::new(x, y + 120, bar_w, bar_h);

        queue.push(QuadInstance::rounded_box(
            bar_rect,
            8.0,
            [0.15, 0.16, 0.20, 0.80],
            [0.26, 0.28, 0.34, 0.60],
            1.0,
            0.55,
        ));

        // Filled progress bar
        let filled_w = ((bar_w as f32) * self.install_progress).max(12.0) as u32;
        let fill_rect = Rect::new(x, y + 120, filled_w, bar_h);
        queue.push(QuadInstance::rounded_box(
            fill_rect,
            8.0,
            [0.25, 0.55, 0.98, 1.0], // Electric blue progress
            [0.45, 0.75, 1.00, 0.80],
            1.0,
            0.56,
        ));

        let pct_text = format!("{:.0}%", self.install_progress * 100.0);
        atlas.layout_text(
            &pct_text,
            Point::new(x + (bar_w as i32 / 2) - 16, y + 158),
            13,
            FontWeight::SemiBold,
            [0.85, 0.88, 0.95, 1.0],
            glyphs,
        );
    }

    fn render_complete(
        &self,
        x: i32,
        y: i32,
        atlas: &InterAtlas,
        _queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        atlas.layout_text(
            "Configuration Terminée !",
            Point::new(x, y + 36),
            22,
            FontWeight::Bold,
            [0.40, 0.85, 0.50, 1.0], // Success green
            glyphs,
        );

        atlas.layout_text(
            "Votre système Gastinel Alpine est prêt à l'emploi.",
            Point::new(x, y + 70),
            15,
            FontWeight::SemiBold,
            [0.96, 0.97, 1.0, 1.0],
            glyphs,
        );

        let lines = [
            "✔ Noyau Linux & modules matériels configurés",
            "✔ Permissions GPU Vulkan et entrées configurées avec seatd",
            "✔ Optimisation mémoire zram et swappiness 10 activées (< 100 Mo RAM)",
            "✔ Direction artistique Inter et compositeur prêts pour le démarrage",
        ];

        for (i, line) in lines.iter().enumerate() {
            atlas.layout_text(
                line,
                Point::new(x + 10, y + 110 + (i as i32) * 28),
                13,
                FontWeight::Regular,
                [0.75, 0.80, 0.88, 1.0],
                glyphs,
            );
        }
    }

    fn render_navigation_buttons(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        let btn_y = self.card_bounds.bottom() - 60;
        let btn_h = 40;

        // "Suivant" or "Terminer" Button (Right)
        let next_text = if self.current_step == WizardStep::Complete {
            "Lancer Gastinel Desktop"
        } else {
            "Continuer →"
        };
        let next_w = 190;
        let next_x = self.card_bounds.right() - next_w - 40;
        let next_rect = Rect::new(next_x, btn_y, next_w as u32, btn_h);

        queue.push(QuadInstance::rounded_box(
            next_rect,
            8.0,
            [0.22, 0.48, 0.95, 0.95],
            [0.45, 0.70, 1.00, 0.90],
            1.0,
            0.60,
        ));

        atlas.layout_text(
            next_text,
            Point::new(next_x + 24, btn_y + 26),
            14,
            FontWeight::SemiBold,
            [1.0, 1.0, 1.0, 1.0],
            glyphs,
        );

        // "Précédent" Button (Left)
        if self.current_step != WizardStep::WelcomeLanguage && self.current_step != WizardStep::Complete {
            let prev_w = 120;
            let prev_x = self.card_bounds.x + 40;
            let prev_rect = Rect::new(prev_x, btn_y, prev_w as u32, btn_h);

            queue.push(QuadInstance::rounded_box(
                prev_rect,
                8.0,
                [0.15, 0.16, 0.20, 0.70],
                [0.28, 0.30, 0.36, 0.50],
                1.0,
                0.60,
            ));

            atlas.layout_text(
                "← Retour",
                Point::new(prev_x + 24, btn_y + 26),
                13,
                FontWeight::Medium,
                [0.75, 0.78, 0.85, 1.0],
                glyphs,
            );
        }
    }
}
