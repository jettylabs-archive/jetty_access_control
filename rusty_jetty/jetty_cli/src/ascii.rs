use colored::{Color, Colorize};
use inquire::ui::Color as InquireColor;

const BATS: &str = r#"









   _   ,_,   _
  / `'=) (='` \
 /.-.-.\ /.-.-.\ 
 `      "      `





   _   ,_,   _
  / `'=) (='` \
 /.-.-.\ /.-.-.\ 
 `      "      `
"#;
const BANNER: &str = "
            
            
            
            
                                           
  ▄████████     ███         ███      ▄██   ▄        
  ███    ███ ▀█████████▄ ▀█████████▄ ███   ██▄      
  ███    █▀     ▀███▀▀██    ▀███▀▀██ ███▄▄▄███      
 ▄███▄▄▄         ███   ▀     ███   ▀ ▀▀▀▀▀▀███      
▀▀███▀▀▀         ███         ███     ▄██   ███      
  ███    █▄      ███         ███     ███   ███      
  ███    ███     ███         ███     ███   ███      
  ██████████    ▄████▀      ▄████▀    ▀█████▀       
                                                    
   ▄█          ▄████████ ▀█████████▄     ▄████████    
  ███         ███    ███   ███    ███   ███    ███    
  ███         ███    ███   ███    ███   ███    █▀     
  ███         ███    ███  ▄███▄▄▄██▀    ███           
  ███       ▀███████████ ▀▀███▀▀▀██▄  ▀███████████    
  ███         ███    ███   ███    ██▄          ███    
  ███▌    ▄   ███    ███   ███    ███    ▄█    ███    
  █████▄▄██   ███    █▀  ▄█████████▀   ▄████████▀     
  ▀                                                   ";

const JETTY_J: &str = "
                    ▒    
                    █   ▒    
                    █ ░ ▒  █ 
                    █ ░ ▒  █ 
                    █ ░█▒  ██
                    █ ░█▒  ██
                    █ ░█▒█ ██
                    █ ░█▒█ ██
                    █ ░█▒█ ██
                    █ ██▒█ ██
                    █ ██▒█ ██
                    █ ██▒████
                    ████▒████
                    ████▒████
                    ████▒████
    ▒███            █████████
█████████           █████████
█████████           █████████
██████████         ██████████
 ███████████████████████████ 
  █████████████████████████  
   ██████████████████████    
      █████████████████       
            ";

pub(crate) const JETTY_ORANGE: Color = Color::TrueColor {
    r: 244,
    g: 113,
    b: 36,
};

/// Inquire version
pub(crate) const JETTY_ORANGE_I: InquireColor = InquireColor::Rgb {
    r: 244,
    g: 113,
    b: 36,
};
pub(crate) const JETTY_ORANGE_DARK: InquireColor = InquireColor::Rgb {
    r: 218,
    g: 88,
    b: 11,
};
pub(crate) const JETTY_ACCENT: InquireColor = InquireColor::Rgb {
    r: 183,
    g: 255,
    b: 255,
};

pub(crate) fn print_banner() {
    let ascii = JETTY_J
        .lines()
        .zip(BANNER.lines().zip(BATS.lines()))
        .map(|(j_line, (banner_line, bats_line))| {
            format!("{}  {} {}", j_line, banner_line, bats_line)
        })
        .collect::<Vec<_>>()
        .join("\n");
    println!("\n{}\n", ascii.color(JETTY_ORANGE));
}
