mod db;
mod commands;
mod commands_firma;

use db::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::block_on(async {
        let db_state = match DbState::new().await {
            Ok(state) => {
                println!("✅ Base de datos inicializada correctamente");
                state
            },
            Err(e) => {
                eprintln!("❌ Error inicializando base de datos: {}", e);
                eprintln!("   Causa: {:?}", e);
                std::process::exit(1);
            }
        };
        
        tauri::Builder::default()
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_notification::init())
            .plugin(tauri_plugin_fs::init())
            .plugin(tauri_plugin_store::Builder::default().build())
            .manage(db_state)
            .invoke_handler(tauri::generate_handler![
                commands::get_jardines,
                commands::get_jardin_by_codigo,
                commands::add_jardin,
                commands::get_partidas,
                commands::add_partida,
                commands::get_requerimientos,
                commands::add_requerimiento,
                commands::update_requerimiento,
                commands::actualizar_fecha_recepcion,
                commands::eliminar_fecha_recepcion,
                commands::delete_requerimiento,
                commands::get_recintos,
                commands::get_recintos_by_jardin,
                commands::add_recinto,
                commands::get_ordenes_trabajo,
                commands::get_orden_trabajo_detalle,
                commands::crear_orden_trabajo,
                commands::update_orden_trabajo,
                commands::eliminar_orden_trabajo,
                commands::get_informes_pago,
                commands::get_informe_pago_detalle,
                commands::get_requerimientos_para_informe,
                commands::crear_informe_pago,
                commands::update_informe_pago,
                commands::eliminar_informe_pago,
                commands::get_configuracion,
                commands::update_configuracion,
                commands::clear_all,
                commands::importar_catalogo_json,
                commands::importar_catalogo_csv,
                commands::importar_catalogo_xlsx,
                commands::importar_catalogo_xlsx_bytes,
                commands::importar_base_datos_completa,
                commands_firma::importar_firma,
                commands_firma::get_firma,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
