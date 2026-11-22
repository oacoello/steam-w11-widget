#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Es, // Español
    En, // Inglés
    Ja, // Japonés
    Pt, // Portugués
    Ru, // Ruso
    Zh, // Mandarín
    Fr, // Francés
}

pub struct Strings {
    pub open_steam: &'static str,
    pub refresh_tooltip: &'static str,
    pub play_tooltip_prefix: &'static str,
    pub no_games: &'static str,
    pub prev_page: &'static str,
    pub next_page: &'static str,
    pub page_label: &'static str,
    pub hours_unit: &'static str,
    pub mins_unit: &'static str,
    pub installed_fallback: &'static str,
    pub use_comma_decimal: bool,
}

#[link(name = "Kernel32")]
extern "system" {
    fn GetUserDefaultUILanguage() -> u16;
}

pub fn detect_system_lang() -> Lang {
    let lang_id = unsafe { GetUserDefaultUILanguage() };
    let primary = lang_id & 0x03FF;
    match primary {
        0x0A => Lang::Es, // LANG_SPANISH
        0x11 => Lang::Ja, // LANG_JAPANESE
        0x16 => Lang::Pt, // LANG_PORTUGUESE
        0x19 => Lang::Ru, // LANG_RUSSIAN
        0x04 => Lang::Zh, // LANG_CHINESE
        0x0C => Lang::Fr, // LANG_FRENCH
        0x09 => Lang::En, // LANG_ENGLISH
        _ => Lang::En,
    }
}

pub fn get_strings(lang: Lang) -> Strings {
    match lang {
        Lang::Es => Strings {
            open_steam: "Abrir Steam",
            refresh_tooltip: "Actualizar biblioteca de Steam",
            play_tooltip_prefix: "Ejecutar",
            no_games: "No se encontraron juegos instalados en Steam.",
            prev_page: "<",
            next_page: ">",
            page_label: "Pág.",
            hours_unit: "h",
            mins_unit: "min",
            installed_fallback: "Instalado",
            use_comma_decimal: true,
        },
        Lang::En => Strings {
            open_steam: "Open Steam",
            refresh_tooltip: "Refresh Steam library",
            play_tooltip_prefix: "Launch",
            no_games: "No installed Steam games found.",
            prev_page: "<",
            next_page: ">",
            page_label: "Page",
            hours_unit: "h",
            mins_unit: "min",
            installed_fallback: "Installed",
            use_comma_decimal: false,
        },
        Lang::Ja => Strings {
            open_steam: "Steamを開く",
            refresh_tooltip: "Steamライブラリを更新",
            play_tooltip_prefix: "起動",
            no_games: "インストール済みのSteamゲームが見つかりません。",
            prev_page: "<",
            next_page: ">",
            page_label: "ページ",
            hours_unit: "時間",
            mins_unit: "分",
            installed_fallback: "インストール済み",
            use_comma_decimal: false,
        },
        Lang::Pt => Strings {
            open_steam: "Abrir Steam",
            refresh_tooltip: "Atualizar biblioteca Steam",
            play_tooltip_prefix: "Iniciar",
            no_games: "Nenhum jogo instalado na Steam encontrado.",
            prev_page: "<",
            next_page: ">",
            page_label: "Pág.",
            hours_unit: "h",
            mins_unit: "min",
            installed_fallback: "Instalado",
            use_comma_decimal: true,
        },
        Lang::Ru => Strings {
            open_steam: "Открыть Steam",
            refresh_tooltip: "Обновить библиотеку Steam",
            play_tooltip_prefix: "Запустить",
            no_games: "Установленные игры Steam не найдены.",
            prev_page: "<",
            next_page: ">",
            page_label: "Стр.",
            hours_unit: "ч",
            mins_unit: "мин",
            installed_fallback: "Установлено",
            use_comma_decimal: true,
        },
        Lang::Zh => Strings {
            open_steam: "打开 Steam",
            refresh_tooltip: "刷新 Steam 游戏库",
            play_tooltip_prefix: "启动",
            no_games: "未找到已安装的 Steam 游戏。",
            prev_page: "<",
            next_page: ">",
            page_label: "页",
            hours_unit: "小时",
            mins_unit: "分钟",
            installed_fallback: "已安装",
            use_comma_decimal: false,
        },
        Lang::Fr => Strings {
            open_steam: "Ouvrir Steam",
            refresh_tooltip: "Actualiser la bibliothèque Steam",
            play_tooltip_prefix: "Lancer",
            no_games: "Aucun jeu Steam installé trouvé.",
            prev_page: "<",
            next_page: ">",
            page_label: "Page",
            hours_unit: "h",
            mins_unit: "min",
            installed_fallback: "Installé",
            use_comma_decimal: true,
        },
    }
}
