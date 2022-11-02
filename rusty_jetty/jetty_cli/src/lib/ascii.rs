use colored::{Color, Colorize};
use inquire::ui::Color as InquireColor;
const BANNER: &str = "
      ██╗███████╗████████╗████████╗██╗   ██╗ 
      ██║██╔════╝╚══██╔══╝╚══██╔══╝╚██╗ ██╔╝ 
      ██║█████╗     ██║      ██║    ╚████╔╝  
 ██   ██║██╔══╝     ██║      ██║     ╚██╔╝   
 ╚█████╔╝███████╗   ██║      ██║      ██║    
  ╚════╝ ╚══════╝   ╚═╝      ╚═╝      ╚═╝    
                                     
                            ██╗      █████╗ ██████╗ ███████╗
                            ██║     ██╔══██╗██╔══██╗██╔════╝
                            ██║     ███████║██████╔╝███████╗
                            ██║     ██╔══██║██╔══██╗╚════██║
                            ███████╗██║  ██║██████╔╝███████║
                            ╚══════╝╚═╝  ╚═╝╚═════╝ ╚══════╝";

const JETTY_J: &str = "                       █[0m
                     █ █[0m
                 █   █ █[0m
                 █   █ █[0m
                 █ █ █ █[0m
                 █ █ █ █[0m
                 ███ ███[0m
                 ███ ███[0m
                 ███████[0m
                 ███████[0m
                 ███████[0m
                 ███████[0m
▓███████         ███████[0m
 ███████         ███████[0m
 ████████       ████████[0m
  █████████████████████ [0m
   ███████████████████  [0m
     ███████████████    [0m
[0m

";

pub(crate) const JETTY_ORANGE: Color = Color::TrueColor {
    r: 244,
    g: 113,
    b: 36,
};

/// Inquire colors
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
    println!("\n{}\n", BANNER.color(JETTY_ORANGE));
}
