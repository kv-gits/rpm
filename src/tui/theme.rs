use ratatui::style::{Color, Modifier, Style};

/// Централизованная система тем для TUI
pub struct Theme {
    // Основные цвета
    pub bg: Color,
    pub surface: Color,
    pub fg: Color,
    pub dimmed: Color,
    pub title: Color,  // Мягкий цвет для заголовков
    
    // Акценты
    pub accent: Color,
    pub accent_secondary: Color,
    
    // Границы
    pub border_inactive: Color,
    pub border_active: Color,
    
    // Выделение
    pub selection_bg: Color,
    pub selection_fg: Color,
    
    // Статус бар
    pub status_bar: Color,
    
    // Специальные цвета
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Theme {
    /// Стиль "Textual / Modern Web" - глубокий темный фон с яркими зелеными акцентами
    pub fn textual_dark() -> Self {
        Self {
            bg: Color::Rgb(17, 17, 17),           // #111111 - очень темный серый
            surface: Color::Rgb(30, 30, 30),      // #1E1E1E - для модальных окон
            fg: Color::Rgb(200, 200, 200),       // #C8C8C8 - мягкий светло-серый текст (было #E0E0E0)
            dimmed: Color::Rgb(117, 117, 117),    // #757575 - приглушенный текст
            title: Color::Rgb(180, 180, 180),    // #B4B4B4 - мягкий цвет для заголовков
            accent: Color::Rgb(0, 255, 95),      // #00FF5F - ядовито-зеленый (Textual Green)
            accent_secondary: Color::Rgb(255, 0, 95), // #FF005F - ярко-розовый
            border_inactive: Color::Rgb(60, 60, 60),  // #3C3C3C
            border_active: Color::Rgb(0, 255, 95),    // #00FF5F
            selection_bg: Color::Rgb(30, 40, 50),    // Темно-синий для выделения
            selection_fg: Color::Rgb(0, 255, 95),    // Зеленый текст при выделении
            status_bar: Color::Rgb(40, 40, 40),      // #282828
            success: Color::Rgb(0, 255, 95),         // Зеленый для успеха
            warning: Color::Rgb(255, 200, 0),        // Желтый для предупреждений
            error: Color::Rgb(255, 0, 95),           // Розовый для ошибок
        }
    }

    /// Стиль "VS Code Dark+ / One Dark" - классический стиль IDE
    pub fn vscode_style() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 30),            // #1E1E1E - классический фон VS Code
            surface: Color::Rgb(37, 37, 38),       // #252526 - фон сайдбара
            fg: Color::Rgb(200, 200, 200),         // #C8C8C8 - мягкий основной текст (было #D4D4D4)
            dimmed: Color::Rgb(113, 113, 113),    // #717171 - приглушенный текст
            title: Color::Rgb(190, 190, 190),     // #BEBEBE - мягкий цвет для заголовков
            accent: Color::Rgb(0, 122, 204),      // #007ACC - Brand Blue
            accent_secondary: Color::Rgb(198, 134, 192), // #C586C0 - мягкий фиолетовый
            border_inactive: Color::Rgb(70, 70, 70),    // #464646
            border_active: Color::Rgb(0, 122, 204),     // #007ACC
            selection_bg: Color::Rgb(38, 79, 120),      // #264F78 - темно-синий для выделения
            selection_fg: Color::Rgb(255, 255, 255),     // Белый текст при выделении
            status_bar: Color::Rgb(25, 25, 26),         // #19191A - приглушенный темный фон для футера
            success: Color::Rgb(106, 153, 85),          // #6A9955 - мягкий зеленый
            warning: Color::Rgb(198, 134, 192),          // #C586C0 - мягкий фиолетовый
            error: Color::Rgb(244, 63, 94),              // #F43F5E - мягкий красный
        }
    }

    /// Стиль "OpenCode / Dark Modern" - нейтральный, современный вид
    pub fn opencode_style() -> Self {
        Self {
            bg: Color::Rgb(24, 25, 38),            // Темный серо-синий (Catppuccin Base)
            surface: Color::Rgb(30, 32, 48),        // Чуть светлее для поверхностей
            fg: Color::Rgb(190, 198, 230),         // #BEC6E6 - более мягкий текст (было #CAD3F5)
            dimmed: Color::Rgb(165, 173, 206),     // #A5ADCE - приглушенный текст
            title: Color::Rgb(180, 188, 220),      // #B4BCDC - мягкий цвет для заголовков
            accent: Color::Rgb(138, 173, 244),    // #8AADF4 - мягкий синий
            accent_secondary: Color::Rgb(198, 160, 246), // #C6A0F6 - мягкий фиолетовый
            border_inactive: Color::Rgb(54, 58, 79),     // #363A4F
            border_active: Color::Rgb(138, 173, 244),    // #8AADF4
            selection_bg: Color::Rgb(54, 58, 79),        // #363A4F - Surface Highlight
            selection_fg: Color::Rgb(202, 211, 245),     // #CAD3F5
            status_bar: Color::Rgb(30, 32, 48),           // #1E2030
            success: Color::Rgb(166, 218, 149),           // #A6DA95 - мягкий зеленый
            warning: Color::Rgb(250, 179, 135),           // #FAB387 - мягкий оранжевый
            error: Color::Rgb(237, 135, 150),              // #ED8796 - мягкий красный
        }
    }

    /// Получить стиль для основного фона
    pub fn bg_style(&self) -> Style {
        Style::default().bg(self.bg)
    }

    /// Получить стиль для поверхности (модальные окна, панели)
    pub fn surface_style(&self) -> Style {
        Style::default().bg(self.surface).fg(self.fg)
    }

    /// Получить стиль для основного текста
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.fg)
    }

    /// Получить стиль для приглушенного текста
    pub fn dimmed_style(&self) -> Style {
        Style::default().fg(self.dimmed)
    }

    /// Получить стиль для активной границы
    pub fn active_border_style(&self) -> Style {
        Style::default().fg(self.border_active)
    }

    /// Получить стиль для неактивной границы
    pub fn inactive_border_style(&self) -> Style {
        Style::default().fg(self.border_inactive)
    }

    /// Получить стиль для выделения
    pub fn selection_style(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .fg(self.selection_fg)
            .add_modifier(Modifier::BOLD)
    }

    /// Получить стиль для акцента
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Получить стиль для активного поля ввода
    pub fn active_input_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Получить стиль для неактивного поля ввода
    pub fn inactive_input_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Получить стиль для заголовка (более мягкий цвет)
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.title)
            .add_modifier(Modifier::BOLD)
    }

    /// Получить стиль для статус бара
    pub fn status_bar_style(&self) -> Style {
        Style::default()
            .bg(self.status_bar)
            .fg(self.fg)
    }

    /// Получить стиль для успешных операций
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Получить стиль для предупреждений
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Получить стиль для ошибок
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }
}

/// Получить тему по имени
pub fn get_theme_by_name(name: &str) -> Theme {
    match name {
        "vscode_style" => Theme::vscode_style(),
        "opencode_style" => Theme::opencode_style(),
        _ => Theme::textual_dark(), // По умолчанию textual_dark
    }
}

/// Глобальная тема по умолчанию (можно изменить на другую)
pub fn default_theme() -> Theme {
    Theme::textual_dark()
}

