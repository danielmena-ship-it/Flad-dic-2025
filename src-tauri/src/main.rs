// Ocultar consola en Windows (release)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sistema_piloto_cont_mant_lib::run()
}
