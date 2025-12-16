# DIAGNÓSTICO: Recepción no guarda / Lista vacía

## PROBLEMA IDENTIFICADO

**Logs muestran**: `update_requerimiento` se ejecuta, pero después `getRequerimientosConRecepcion()` retorna array vacío.

**Root cause**: El comando `update_requerimiento` en Rust construye query dinámica pero puede tener conflicto en orden de bindings cuando se combinan parámetros.

## SOLUCIONES

### Fix 1: Agregar logs en Rust para debugging

Modificar `src-tauri/src/commands.rs` línea 216:

```rust
// ANTES:
query.bind(id)
    .execute(&*db.pool)
    .await
    .map_err(|e| e.to_string())?;

// DESPUÉS:  
let result = query.bind(id)
    .execute(&*db.pool)
    .await
    .map_err(|e| {
        println!("❌ ERROR update_requerimiento: {}", e);
        e.to_string()
    })?;
    
println!("✅ update_requerimiento ejecutado: {} filas afectadas", result.rows_affected());
```

### Fix 2: Verificar transformación snake_case

Modificar `src/lib/api/tauri.js` línea 60:

```javascript
// VERIFICAR que el parámetro se envía correcto:
update: (id, data) => {
  const params = { id };
  if (data.fechaRecepcion !== undefined) params.fecha_recepcion = data.fechaRecepcion;
  // ...otros params
  console.log('📤 [TAURI-API] Enviando:', JSON.stringify(params, null, 2));
  return invoke('update_requerimiento', params).then(result => {
    console.log('📥 [TAURI-API] Respuesta:', result);
    return result;
  });
}
```

### Fix 3: Forzar recarga en ListaRecepcion

Modificar `src/lib/components/RecepcionIngreso.svelte` línea 145:

```javascript
async function guardar() {
  // ... validaciones ...
  try {
    cargando = true;
    await guardarFechasRecepcion(seleccionadosConFecha);
    
    // NUEVO: Forzar delay para que commit de DB termine
    await new Promise(resolve => setTimeout(resolve, 500));
    
    mensaje = `✅ ${seleccionadosConFecha.length} recepción(es) guardada(s)`;
    await cargarRequerimientos();
    setTimeout(() => mensaje = '', 5000);
  } catch (error) {
    mensaje = '❌ Error al guardar: ' + error.message;
  } finally {
    cargando = false;
  }
}
```

## PLAN DE REPARACIÓN

1. Agregar logs detallados en Rust (Fix 1)
2. Recompilar: `cd src-tauri && cargo build --release`
3. Reiniciar app y probar guardar recepción
4. Revisar console logs para ver si hay errores SQL
5. Si persiste, aplicar Fix 2 y Fix 3
