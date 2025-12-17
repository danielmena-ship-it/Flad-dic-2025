// TEMPORAL: Habilitar consola para debug en Windows
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    println!("🔷 [MAIN] Iniciando desde main.rs...");
    sistema_piloto_cont_mant_lib::run()
}
