<script>
  import { onMount } from 'svelte';
  import { writable } from 'svelte/store';
  import { setContext } from 'svelte';
  import { db } from '$lib/api/tauri';
  import { setDbReady, setDbError } from '$lib/stores/db';
  import { configuracion } from '$lib/stores/configuracion';
  import Bienvenida from '$lib/components/Bienvenida.svelte';
  import ToastContainer from '$lib/components/ToastContainer.svelte';
  import ModalITO from '$lib/components/ModalITO.svelte';
  import { toast } from '$lib/utils/toast';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile, writeFile } from '@tauri-apps/plugin-fs';
  import { invalidarCatalogos, cargarJardines, cargarPartidas } from '$lib/stores/catalogos';
  import '../app.css';

  const tabActual = writable('ingreso');
  setContext('tabActual', tabActual);
  
  let mostrarBienvenida = false;
  let inicializado = false;
  let titulo = 'FLAD';
  let inputCatalogo;
  let inputBaseDatos;
  let menuImportarAbierto = false;
  let menuExportarAbierto = false;
  let modalITOVisible = false;

  onMount(async () => {
    const startTime = Date.now();
    console.log('🔷 [' + startTime + '] Inicializando aplicación...');
    alert('DEBUG: Inicio onMount - ' + new Date().toISOString());
    
    try {
      console.log('⏳ Cargando configuración...');
      alert('DEBUG: Antes de configuracion.cargar()');
      await configuracion.cargar();
      console.log('✅ Configuración cargada en ' + (Date.now() - startTime) + 'ms');
      alert('DEBUG: Config cargada OK');
    } catch (err) {
      const msg = 'Error config: ' + err;
      console.error('⚠️ ' + msg);
      alert('DEBUG ERROR: ' + msg);
    }
    
    try {
      console.log('⏳ Obteniendo título...');
      alert('DEBUG: Antes de db.configuracion.get()');
      
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Timeout título')), 3000)
      );
      
      const configPromise = db.configuracion.get();
      const config = await Promise.race([configPromise, timeoutPromise]);
      
      titulo = config.titulo || 'FLAD';
      console.log('✅ Título: ' + titulo);
      alert('DEBUG: Título obtenido: ' + titulo);
    } catch (err) {
      const msg = 'Error título: ' + err;
      console.error('⚠️ ' + msg);
      alert('DEBUG ERROR: ' + msg);
      titulo = 'FLAD';
    }
    
    setDbReady(true);
    inicializado = true;
    const totalTime = Date.now() - startTime;
    console.log('✅ Aplicación lista en ' + totalTime + 'ms');
    alert('DEBUG: App lista en ' + totalTime + 'ms');

    const handleClickOutside = (event) => {
      if (menuImportarAbierto && !event.target.closest('.dropdown-importar')) {
        menuImportarAbierto = false;
      }
      if (menuExportarAbierto && !event.target.closest('.dropdown-exportar')) {
        menuExportarAbierto = false;
      }
    };

    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });

  function cambiarTab(tab) {
    tabActual.set(tab);
  }

  function toggleMenuImportar() {
    menuImportarAbierto = !menuImportarAbierto;
    if (menuImportarAbierto) menuExportarAbierto = false;
  }

  function toggleMenuExportar() {
    menuExportarAbierto = !menuExportarAbierto;
    if (menuExportarAbierto) menuImportarAbierto = false;
  }

  async function handleImportarExcel() {
    menuImportarAbierto = false;
    if (!inputCatalogo) {
      toast.error('Error: Input no inicializado');
      return;
    }
    inputCatalogo.click();
  }

  async function handleImportarBaseDatos() {
    menuImportarAbierto = false;
    inputBaseDatos.click();
  }

  async function handleImportarFirma() {
    menuImportarAbierto = false;
    modalITOVisible = true;
  }

  async function handleITOGuardado(event) {
    configuracion.actualizarITO(event.detail.nombre, event.detail.firma?.split(',')[1]);
    toast.success('✅ Datos ITO guardados correctamente');
  }

  async function handleCatalogoSelected(event) {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      const arrayBuffer = await file.arrayBuffer();
      const bytes = Array.from(new Uint8Array(arrayBuffer));
      
      const result = await db.importar.catalogoXlsxBytes(bytes);
      
      // Recargar stores con force=true - esto actualiza el UI inmediatamente
      await cargarJardines(true);
      await cargarPartidas(true);
      
      // Recargar configuración (título)
      if (result.contrato) {
        const config = await db.configuracion.get();
        titulo = config.titulo || 'FLAD';
        await configuracion.cargar(); // ✅ Recargar store
      }
      
      const msg = `✅ Importado: ${result.jardines} jardines, ${result.partidas} partidas, ${result.recintos} recintos${result.contrato ? ', contrato actualizado' : ''}`;
      toast.success(msg);
      inputCatalogo.value = '';
    } catch (err) {
      console.error('❌ Error:', err);
      toast.error('Error: ' + (err.message || err));
      inputCatalogo.value = '';
    }
  }

  async function handleBaseDatosSelected(event) {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      const result = await db.importar.baseDatosCompleta(text);
      
      toast.success(result + ' - Recargando aplicación...');
      
      // ✅ Forzar recarga completa para limpiar todos los cachés
      setTimeout(() => location.reload(), 1000);
    } catch (err) {
      console.error('❌ Error:', err);
      toast.error('Error: ' + (err.message || err));
      inputBaseDatos.value = '';
    }
  }

  async function handleExportarJSON() {
    menuExportarAbierto = false;
    try {
      // Obtener todos los datos
      const jardines = await db.jardines.getAll();
      const partidas = await db.partidas.getAll();
      const recintos = await db.recintos.getAll();
      const requerimientosEnriquecidos = await db.requerimientos.getAll();
      const ordenesTrabajo = await db.ordenesTrabajo.getAll();
      const informesPago = await db.informesPago.getAll();
      const config = await db.configuracion.get();
      
      // Obtener firma base64 (sin prefijo data:image)
      const firmaBase64 = await db.importar.getFirma();
      
      // Convertir RequerimientoEnriquecido → Requerimiento base (formato para importación)
      // ✅ Exportar CÓDIGOS en lugar de IDs para evitar FOREIGN KEY errors
      const requerimientos = requerimientosEnriquecidos.map(req => {
        const ot = req.otId ? ordenesTrabajo.find(o => o.id === req.otId) : null;
        const informe = req.informePagoId ? informesPago.find(ip => ip.id === req.informePagoId) : null;
        
        // Calcular campos financieros (con o sin fecha_recepcion)
        const a_pago = req.aPago ?? req.precioTotal;
        const utilidades = req.utilidades ?? (a_pago * config.porcentajeUtilidades);
        const subtotal = a_pago + utilidades;
        const iva = req.iva ?? (subtotal * 0.19);
        const total_linea = req.totalLinea ?? (a_pago + utilidades + iva);
        
        return {
          jardin_codigo: req.jardinCodigo,
          recinto: req.recinto,
          partida_item: req.partidaItem,
          cantidad: req.cantidad,
          precio_unitario: req.cantidad > 0 ? req.precioTotal / req.cantidad : 0,
          precio_total: req.precioTotal,
          fecha_inicio: req.fechaInicio,
          fecha_registro: req.fechaRegistro,
          estado: req.estado,
          ot_codigo: ot?.codigo || null,  // ✅ Código en lugar de ID
          informe_codigo: informe?.codigo || null,  // ✅ Código en lugar de ID
          fecha_recepcion: req.fechaRecepcion,
          plazo_dias: req.plazoDias,
          plazo_adicional: req.plazoAdicional,
          plazo_total: req.plazoTotal,
          fecha_limite: req.fechaLimite,
          multa: req.multa,
          a_pago: a_pago,
          utilidades: utilidades,
          iva: iva,
          total_linea: total_linea,
          descripcion: req.descripcion,
          observaciones: req.observaciones
        };
      });
      
      // Convertir camelCase a snake_case para otros datos
      const toSnakeCase = (obj) => {
        if (Array.isArray(obj)) return obj.map(toSnakeCase);
        if (obj === null || typeof obj !== 'object') return obj;
        
        return Object.keys(obj).reduce((acc, key) => {
          const snakeKey = key.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`);
          acc[snakeKey] = toSnakeCase(obj[key]);
          return acc;
        }, {});
      };
      
      // Estructura exacta que espera importar_base_datos_completa
      const data = {
        jardines: toSnakeCase(jardines),
        partidas: toSnakeCase(partidas),
        recintos: toSnakeCase(recintos),
        requerimientos: requerimientos, // Ya incluye informe_id
        ordenes_trabajo: toSnakeCase(ordenesTrabajo),
        informes_pago: toSnakeCase(informesPago),
        configuracion: {
          titulo: config.titulo,
          contratista: config.contratista,
          prefijo_correlativo: config.prefijoCorrelativo,
          ito_nombre: config.itoNombre || null,
          porcentaje_utilidades: config.porcentajeUtilidades || 0.25,
          firma_png_base64: firmaBase64 || null
        }
      };
      
      const json = JSON.stringify(data, null, 2);
      
      // Mostrar diálogo de guardado
      const filePath = await save({
        defaultPath: `flad_backup_${new Date().toISOString().split('T')[0]}.json`,
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });
      
      if (!filePath) return; // Usuario canceló
      
      await writeTextFile(filePath, json);
      
      toast.success(`✅ Exportado: ${requerimientos.length} requerimientos, ${ordenesTrabajo.length} OTs, ${informesPago.length} informes`);
    } catch (err) {
      console.error('Error exportando JSON:', err);
      toast.error('Error al exportar JSON: ' + (err.message || err));
    }
  }

  async function handleExportarExcel() {
    menuExportarAbierto = false;
    try {
      // Importar XLSX desde npm
      const XLSX = await import('xlsx');
      
      const jardines = await db.jardines.getAll();
      const partidas = await db.partidas.getAll();
      const recintos = await db.recintos.getAll();
      const requerimientos = await db.requerimientos.getAll();
      const config = await db.configuracion.get();
      const ordenesTrabajo = await db.ordenesTrabajo.getAll();
      const informesPago = await db.informesPago.getAll();
      
      // Obtener relación informe-requerimientos
      const informeReqMap = new Map();
      for (const informe of informesPago) {
        try {
          const detalle = await db.informesPago.getDetalle(informe.id);
          if (detalle && Array.isArray(detalle)) {
            detalle.forEach(req => {
              informeReqMap.set(req.id, informe.id);
            });
          }
        } catch (err) {
          console.warn(`No se pudo obtener detalle del informe ${informe.id}:`, err);
        }
      }
      
      // Crear workbook con múltiples hojas
      const wb = XLSX.utils.book_new();
      
      // Hoja: Jardines
      const wsJardines = XLSX.utils.json_to_sheet(jardines.map(j => ({
        ID: j.id,
        Código: j.codigo,
        Nombre: j.nombre,
        'Fecha Creación': j.createdAt
      })));
      XLSX.utils.book_append_sheet(wb, wsJardines, 'Jardines');
      
      // Hoja: Partidas
      const wsPartidas = XLSX.utils.json_to_sheet(partidas.map(p => ({
        ID: p.id,
        Item: p.item,
        Partida: p.partida,
        Unidad: p.unidad,
        'Precio Unitario': p.precioUnitario,
        'Fecha Creación': p.createdAt
      })));
      XLSX.utils.book_append_sheet(wb, wsPartidas, 'Partidas');
      
      // Hoja: Recintos
      const wsRecintos = XLSX.utils.json_to_sheet(recintos.map(r => ({
        ID: r.id,
        'Código Jardín': r.jardinCodigo,
        Jardín: jardines.find(j => j.codigo === r.jardinCodigo)?.nombre || '',
        Nombre: r.nombre,
        'Fecha Creación': r.createdAt
      })));
      XLSX.utils.book_append_sheet(wb, wsRecintos, 'Recintos');
      
      // Hoja: Requerimientos - Solo códigos OT e IP (no IDs)
      const reqsEnriquecidos = requerimientos.map(req => {
        const jardin = jardines.find(j => j.codigo === req.jardinCodigo);
        
        // Obtener código de OT
        const ot = req.otId ? ordenesTrabajo.find(o => o.id === req.otId) : null;
        
        // Obtener código de Informe
        const informeId = informeReqMap.get(req.id) || req.informePagoId;
        const informe = informeId ? informesPago.find(ip => ip.id === informeId) : null;
        
        // Calcular precio unitario
        const precioUnitario = req.cantidad > 0 ? req.precioTotal / req.cantidad : 0;
        
        // Calcular campos financieros (con o sin fecha_recepcion)
        const a_pago = req.aPago ?? req.precioTotal;
        const utilidades = req.utilidades ?? (a_pago * config.porcentajeUtilidades);
        const subtotal = a_pago + utilidades;
        const iva = req.iva ?? (subtotal * 0.19);
        const total_linea = req.totalLinea ?? (a_pago + utilidades + iva);
        
        return {
          'ID': req.id,
          'Código Jardín': req.jardinCodigo,
          'Jardín': jardin?.nombre || '',
          'Recinto': req.recinto || '',
          'Partida Item': req.partidaItem,
          'Partida Nombre': req.partidaNombre || '',
          'Unidad': req.partidaUnidad || '',
          'Cantidad': req.cantidad,
          'Precio Unitario': precioUnitario,
          'Precio Total': req.precioTotal,
          'Fecha Inicio': req.fechaInicio,
          'Fecha Registro': req.fechaRegistro,
          'Plazo (días)': req.plazoDias,
          'Plazo Adicional': req.plazoAdicional,
          'Plazo Total': req.plazoTotal,
          'Fecha Límite': req.fechaLimite || '',
          'Fecha Recepción': req.fechaRecepcion || '',
          'Días Atraso': req.diasAtraso || 0,
          'Multa': req.multa,
          'A Pago': a_pago,
          'Utilidades': utilidades,
          'IVA': iva,
          'Total Linea': total_linea,
          'Estado': req.estado,
          'OT Código': ot?.codigo || '',
          'IP Código': informe?.codigo || '',
          'Descripción': req.descripcion || '',
          'Observaciones': req.observaciones || ''
        };
      });
      const wsRequerimientos = XLSX.utils.json_to_sheet(reqsEnriquecidos);
      XLSX.utils.book_append_sheet(wb, wsRequerimientos, 'Requerimientos');
      
      // Hoja: Órdenes de Trabajo (sin observaciones)
      const otsEnriquecidas = ordenesTrabajo.map(ot => {
        const jardin = jardines.find(j => j.codigo === ot.jardinCodigo);
        const reqsAsociados = requerimientos.filter(r => r.otId === ot.id);
        
        return {
          ID: ot.id,
          Código: ot.codigo,
          'Código Jardín': ot.jardinCodigo,
          Jardín: jardin?.nombre || '',
          'Fecha Creación': ot.fechaCreacion,
          'Cantidad Requerimientos': reqsAsociados.length
        };
      });
      const wsOT = XLSX.utils.json_to_sheet(otsEnriquecidas);
      XLSX.utils.book_append_sheet(wb, wsOT, 'Órdenes de Trabajo');
      
      // Hoja: Informes de Pago (sin observaciones, conteo correcto)
      const ipsEnriquecidos = informesPago.map(ip => {
        const jardin = jardines.find(j => j.codigo === ip.jardinCodigo);
        // Contar usando el mapa que creamos antes
        const cantidadReqs = Array.from(informeReqMap.entries())
          .filter(([reqId, infId]) => infId === ip.id).length;
        
        return {
          ID: ip.id,
          Código: ip.codigo,
          'Código Jardín': ip.jardinCodigo,
          Jardín: jardin?.nombre || '',
          'Fecha Creación': ip.fechaCreacion,
          Neto: ip.neto,
          Utilidades: ip.utilidades,
          IVA: ip.iva,
          'Total Pagar': ip.totalPagar,
          'Cantidad Requerimientos': cantidadReqs
        };
      });
      const wsIP = XLSX.utils.json_to_sheet(ipsEnriquecidos);
      XLSX.utils.book_append_sheet(wb, wsIP, 'Informes de Pago');
      
      // Generar archivo Excel como array buffer
      const wbout = XLSX.write(wb, { bookType: 'xlsx', type: 'array' });
      
      // Mostrar diálogo de guardado
      const filePath = await save({
        defaultPath: `flad_reporte_${new Date().toISOString().split('T')[0]}.xlsx`,
        filters: [{
          name: 'Excel',
          extensions: ['xlsx']
        }]
      });
      
      if (!filePath) return; // Usuario canceló
      
      await writeFile(filePath, new Uint8Array(wbout));
      
      toast.success('✅ Excel exportado correctamente');
    } catch (err) {
      console.error('Error exportando Excel:', err);
      toast.error('Error al exportar Excel: ' + (err.message || err));
    }
  }

  function toggleBienvenida() {
    mostrarBienvenida = !mostrarBienvenida;
  }
</script>

{#if !inicializado}
  <div class="loading">Inicializando sistema...</div>
{:else if mostrarBienvenida}
  <Bienvenida onContinue={() => mostrarBienvenida = false} />
{:else}
  <div class="app">
    <header>
      <div class="header-top">
        <div class="header-left">
          <button on:click={toggleBienvenida} class="btn-flad" title="Información del sistema">
            FLAD
          </button>
          <h1>{titulo}</h1>
        </div>
        
        <div class="actions">
          <div class="dropdown dropdown-importar">
            <button on:click={toggleMenuImportar} class="btn-secondary">
              Importar ▾
            </button>
            {#if menuImportarAbierto}
              <div class="dropdown-menu">
                <button on:click={handleImportarExcel} class="dropdown-item">
                  Catálogo
                </button>
                <button on:click={handleImportarBaseDatos} class="dropdown-item">
                  Base de Datos
                </button>
                <button on:click={handleImportarFirma} class="dropdown-item">
                  Firma
                </button>
              </div>
            {/if}
          </div>
          
          <div class="dropdown dropdown-exportar">
            <button on:click={toggleMenuExportar} class="btn-secondary">
              Exportar ▾
            </button>
            {#if menuExportarAbierto}
              <div class="dropdown-menu">
                <button on:click={handleExportarJSON} class="dropdown-item">
                  JSON
                </button>
                <button on:click={handleExportarExcel} class="dropdown-item">
                  Excel
                </button>
              </div>
            {/if}
          </div>
        </div>
      </div>
      
      <nav>
        <div class="nav-group">
          <span class="group-title">Requerimiento</span>
          <button 
            on:click={() => cambiarTab('ingreso')} 
            class:active={$tabActual === 'ingreso'}
          >
            Ingreso
          </button>
          <button 
            on:click={() => cambiarTab('listado')} 
            class:active={$tabActual === 'listado'}
          >
            Lista
          </button>
        </div>
        
        <div class="nav-group">
          <span class="group-title">Orden de Trabajo</span>
          <button 
            on:click={() => cambiarTab('crear-ot')} 
            class:active={$tabActual === 'crear-ot'}
          >
            Ingreso
          </button>
          <button 
            on:click={() => cambiarTab('lista-ot')} 
            class:active={$tabActual === 'lista-ot'}
          >
            Lista
          </button>
        </div>
        
        <div class="nav-group">
          <span class="group-title">Recepción</span>
          <button 
            on:click={() => cambiarTab('recepcion')} 
            class:active={$tabActual === 'recepcion'}
          >
            Ingreso
          </button>
          <button 
            on:click={() => cambiarTab('lista-recepcion')} 
            class:active={$tabActual === 'lista-recepcion'}
          >
            Lista
          </button>
        </div>
        
        <div class="nav-group">
          <span class="group-title">Informe de Pago</span>
          <button 
            on:click={() => cambiarTab('ingresar-pago')} 
            class:active={$tabActual === 'ingresar-pago'}
          >
            Ingreso
          </button>
          <button 
            on:click={() => cambiarTab('lista-pago')} 
            class:active={$tabActual === 'lista-pago'}
          >
            Lista
          </button>
        </div>
      </nav>
    </header>
    
    <main>
      <slot />
    </main>
  </div>
{/if}

<input 
  type="file" 
  accept=".xlsx" 
  bind:this={inputCatalogo} 
  on:change={handleCatalogoSelected} 
  style="display: none" 
/>

<input 
  type="file" 
  accept=".json,application/json" 
  bind:this={inputBaseDatos} 
  on:change={handleBaseDatosSelected} 
  style="display: none" 
/>

<ModalITO 
  bind:visible={modalITOVisible}
  nombreActual={$configuracion.itoNombre || ''}
  firmaActual={$configuracion.firmaBase64 ? `data:image/png;base64,${$configuracion.firmaBase64}` : null}
  on:guardado={handleITOGuardado}
/>

<ToastContainer />

<style>
  :global(body) {
    margin: 0;
    padding: 0;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    color: var(--text-secondary);
    font-family: 'Inter', sans-serif;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  header {
    background: var(--bg-card);
    border-bottom: 1px solid var(--border);
    padding: 1rem;
  }

  .header-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  h1 {
    margin: 0;
    color: var(--accent);
    font-size: 1.5rem;
    font-weight: 600;
  }

  nav {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .nav-group {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: auto 1fr;
    gap: 0.25rem;
    padding: 0.5rem;
    background: rgba(45, 62, 80, 0.3);
    border-radius: 0.5rem;
    border: 1px solid var(--border);
  }

  .group-title {
    grid-column: 1 / -1;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 0 0.5rem 0.25rem 0.5rem;
    border-bottom: 1px solid var(--border);
    margin-bottom: 0.25rem;
  }

  nav button {
    padding: 0.75rem 1.25rem;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-tertiary);
    border-radius: 0.5rem;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
    font-weight: 500;
  }

  nav button:hover {
    background: var(--bg-card);
    border-color: var(--accent);
    color: var(--text-secondary);
  }

  nav button.active {
    background: var(--btn-primary);
    color: white;
    border-color: var(--btn-primary);
    box-shadow: 0 2px 8px rgba(90, 143, 196, 0.3);
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .dropdown {
    position: relative;
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 0.25rem;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    min-width: 180px;
    overflow: hidden;
  }

  .dropdown-item {
    width: 100%;
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    text-align: left;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
    font-weight: 500;
    font-size: 0.95rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .dropdown-item:hover {
    background: var(--bg-primary);
    color: var(--accent);
  }

  main {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }
</style>
