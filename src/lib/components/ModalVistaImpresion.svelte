<script>
  import { createEventDispatcher, onMount } from 'svelte';
  import { db } from '$lib/api/tauri';
  import { configuracion } from '$lib/stores/configuracion';
  import { formatearNumero, formatearCantidad } from '$lib/utils/calculos.js';
  import { formatearFecha } from '$lib/utils/formatoFecha.js';
  import { enriquecerRequerimientos } from '$lib/utils/enriquecimiento.js';
  import html2pdf from 'html2pdf.js';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeFile } from '@tauri-apps/plugin-fs';

  export let ot;
  export let requerimientos;

  const dispatch = createEventDispatcher();

  let jardinCompleto = null;
  let cargando = true;
  let mensajeGuardado = '';
  let requerimientosEnriquecidos = [];

  // ✅ Recargar datos cada vez que se abre el modal
  $: if (ot) {
    cargarDatos();
  }

  async function cargarDatos() {
    cargando = true;
    const jardines = await db.jardines.getAll();
    jardinCompleto = jardines.find(j => j.codigo === ot.jardinCodigo);
    requerimientosEnriquecidos = await enriquecerRequerimientos(requerimientos);
    cargando = false;
  }

  onMount(async () => {
    await cargarDatos();
  });

  function extraerNumeroZona(zona) {
    const match = zona.match(/^(\d+)/);
    return match ? match[1] : zona;
  }

  async function generarPDF() {
    const elemento = document.querySelector('.contenido-imprimible');
    
    const opciones = {
      margin: [5, 8],
      image: { type: 'jpeg', quality: 0.98 },
      html2canvas: { 
        scale: 2,
        useCORS: true,
        letterRendering: true
      },
      jsPDF: { 
        unit: 'mm', 
        format: 'legal', 
        orientation: 'landscape' 
      },
      pagebreak: { 
        mode: ['css', 'legacy']
      }
    };

    // Generar PDF como ArrayBuffer
    const pdfArrayBuffer = await html2pdf().set(opciones).from(elemento).output('arraybuffer');
    
    // Mostrar diálogo de guardar
    const filePath = await save({
      defaultPath: `${ot.codigo}.pdf`,
      filters: [{
        name: 'PDF',
        extensions: ['pdf']
      }]
    });

    if (filePath) {
      // Convertir ArrayBuffer a Uint8Array
      const pdfBytes = new Uint8Array(pdfArrayBuffer);
      
      // Guardar archivo
      await writeFile(filePath, pdfBytes);
      
      mensajeGuardado = '✅ PDF guardado exitosamente';
      setTimeout(() => mensajeGuardado = '', 3000);
    }
  }

  function cerrar() {
    dispatch('cerrar');
  }
</script>

<div class="overlay" on:click={cerrar}>
  <div class="modal-impresion" on:click|stopPropagation>
    
    {#if cargando}
      <div class="cargando">
        <p>Cargando datos...</p>
      </div>
    {:else}
      <!-- Botones de acción (no se imprimen) -->
      <div class="acciones-modal no-print">
        <button class="btn-cerrar" on:click={cerrar}>✕ Cerrar</button>
        <button class="btn-pdf" on:click={generarPDF}>📄 Descargar PDF</button>
      </div>

      <!-- Contenido imprimible -->
      <div class="contenido-imprimible">
        
        <!-- Encabezado -->
        <div class="encabezado">
          <h1>ORDEN DE TRABAJO</h1>
          <div class="codigo-ot">{ot.codigo}{ot.estado && ot.estado !== 'Inicial' ? ' ' + ot.estado : ''}</div>
        </div>

        <!-- Información General -->
        <div class="seccion-info">
          <div class="info-row">
            <span class="label">Jardín:</span>
            <span class="valor">{ot.jardinCodigo} - {jardinCompleto?.nombre || 'N/A'}</span>
          </div>
          <div class="info-row">
            <span class="label">Fecha Creación:</span>
            <span class="valor">{formatearFecha(ot.fechaCreacion)}</span>
          </div>
          <div class="info-row">
            <span class="label">Contratista:</span>
            <span class="valor">{$configuracion.contratista || 'Sin especificar'}</span>
          </div>
          <div class="info-row">
            <span class="label">ITO:</span>
            <span class="valor">{$configuracion.itoNombre || 'Sin especificar'}</span>
          </div>
        </div>

        <!-- Requerimientos -->
        <div class="seccion-requerimientos">
          <h2>REQUERIMIENTOS INCLUIDOS</h2>
          
          {#if requerimientos.length === 0}
            <p class="sin-datos">No hay requerimientos asignados</p>
          {:else}
            <div class="tabla-requerimientos-wrapper">
              <table class="tabla-requerimientos">
                <thead>
                  <tr>
                    <th>#</th>
                    <th>Zona</th>
                    <th>Item - Partida</th>
                    <th>Cantidad</th>
                    <th>F. Inicio</th>
                    <th>Plazo</th>
                    <th>Plazo Obs.</th>
                    <th>F. Límite</th>
                    <th>Descripción</th>
                    <th>Observaciones</th>
                  </tr>
                </thead>
                <tbody>
                  {#each requerimientosEnriquecidos as req, index}
                    <tr>
                      <td class="centrado">{index + 1}</td>
                      <td>{extraerNumeroZona(req.recinto)}</td>
                      <td>{req.partidaItem} - {req.partidaNombre}</td>
                      <td class="derecha">{formatearCantidad(req.cantidad)} {req.partidaUnidad}</td>
                      <td class="centrado">{formatearFecha(req.fechaInicio)}</td>
                      <td class="centrado">{req.plazoDias || req.plazoTotal || 0} días</td>
                      <td class="centrado">{req.plazoObservacion ? `${req.plazoObservacion} días` : '-'}</td>
                      <td class="centrado">{req.fechaLimite ? formatearFecha(req.fechaLimite) : '-'}</td>
                      <td class="descripcion-cell">{req.descripcion || '-'}</td>
                      <td class="descripcion-cell">{req.observaciones || '-'}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>

        <!-- Total -->
        <div class="seccion-total">
          <div class="total-row">
            <span class="total-label">Total Requerimientos:</span>
            <span class="total-valor">{requerimientos.length}</span>
          </div>
        </div>

        <!-- Firma -->
        <div class="seccion-firma">
          <div class="firma-box">
            {#if $configuracion.firmaBase64}
              <img src={`data:image/png;base64,${$configuracion.firmaBase64}`} alt="Firma ITO" class="firma-imagen" />
            {/if}
            <p class="firma-label">Firma ITO</p>
            <p class="firma-nombre">{$configuracion.itoNombre || 'Sin especificar'}</p>
          </div>
          
          <div class="firma-box">
            <div class="firma-linea"></div>
            <p class="firma-label">Firma Contratista</p>
            <p class="firma-nombre">{$configuracion.contratista || 'Sin especificar'}</p>
          </div>
        </div>

        <!-- Pie de página -->
        <div class="pie-pagina">
          <p>Documento generado el {formatearFecha(new Date())}</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 2000;
    overflow-y: auto;
    padding: 2rem 1rem;
  }

  .modal-impresion {
    background: #ffffff;
    border-radius: 12px;
    max-width: 900px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 4px 30px rgba(0, 0, 0, 0.7);
    position: relative;
  }

  .cargando {
    padding: 3rem;
    text-align: center;
    color: #333;
  }

  /* Botones de acción */
  .acciones-modal {
    position: sticky;
    top: 0;
    background: #f5f5f5;
    border-radius: 12px 12px 0 0;
    padding: 1rem 1.5rem;
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    border-bottom: 2px solid #ddd;
    z-index: 10;
  }

  .acciones-modal button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    font-family: 'Inter', sans-serif;
  }

  .btn-cerrar {
    background: #2d3e50;
    color: #a8c5e0;
  }

  .btn-cerrar:hover {
    background: #3a4f66;
  }

  .btn-imprimir {
    background: #5a8fc4;
    color: white;
  }

  .btn-imprimir:hover {
    background: #4a7fb4;
  }

  .btn-pdf {
    background: #4caf50;
    color: white;
  }

  .btn-pdf:hover {
    background: #45a049;
  }

  /* Aviso de orientación */
  .aviso-orientacion {
    background: #ff9800;
    color: #000;
    padding: 1rem;
    margin: 1rem 2rem;
    border-radius: 8px;
    text-align: center;
    font-size: 1rem;
    font-weight: 500;
    border: 2px solid #f57c00;
  }

  .aviso-orientacion strong {
    font-weight: 700;
    text-decoration: underline;
  }

  /* Contenido imprimible */
  .contenido-imprimible {
    padding: 2rem;
  }

  .encabezado {
    text-align: center;
    margin-bottom: 2rem;
    border-bottom: 3px solid #5a8fc4;
    padding-bottom: 1.5rem;
  }

  .encabezado h1 {
    color: #2c3e50;
    font-size: 2rem;
    margin: 0 0 0.5rem 0;
    font-weight: 700;
  }

  .codigo-ot {
    color: #5a8fc4;
    font-size: 1.5rem;
    font-weight: 600;
  }

  /* Sección Información */
  .seccion-info {
    background: #f5f5f5;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 2rem;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
    border-bottom: 1px solid #999;
  }

  .info-row:last-child {
    border-bottom: none;
  }

  .label {
    color: #000;
    font-weight: 600;
    font-size: 0.9rem;
  }

  .valor {
    color: #000;
    font-size: 0.9rem;
  }

  /* Sección Requerimientos */
  .seccion-requerimientos {
    margin-bottom: 2rem;
  }

  .seccion-requerimientos h2 {
    color: #000;
    font-size: 1.3rem;
    margin-bottom: 1.5rem;
    background: #e0e0e0;
    border-bottom: 2px solid #333;
    padding: 0.5rem 1rem;
  }

  .sin-datos {
    text-align: center;
    color: #8b9eb3;
    padding: 2rem;
    font-style: italic;
  }

  /* Tabla de Requerimientos */
  .tabla-requerimientos-wrapper {
    overflow-x: auto;
    margin-top: 1rem;
  }

  .tabla-requerimientos {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
    page-break-inside: auto;
  }

  .tabla-requerimientos thead th {
    background: #2c3e50;
    color: white;
    padding: 0.75rem;
    text-align: left;
    font-weight: 600;
    border-bottom: 2px solid #5a8fc4;
    border-left: none;
    border-right: none;
    white-space: nowrap;
  }

  .tabla-requerimientos tbody td {
    padding: 0.75rem;
    color: #000;
    border-bottom: 1px solid #ccc;
    border-left: none;
    border-right: none;
    vertical-align: top;
  }

  .tabla-requerimientos tbody tr:hover {
    background: #f5f5f5;
  }

  .tabla-requerimientos tbody tr:last-child td {
    border-bottom: 2px solid #ccc;
  }

  .centrado {
    text-align: center;
  }

  .derecha {
    text-align: right;
  }

  .destacado {
    color: #81c784 !important;
    font-weight: 600;
  }

  .descripcion-cell {
    max-width: 200px;
    word-wrap: break-word;
    white-space: normal;
    font-size: 0.85rem;
    line-height: 1.4;
  }

  /* Sección Total */
  .seccion-total {
    background: #e8f4f8;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    border: 2px solid #333;
    margin-bottom: 2rem;
  }

  .total-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .total-label {
    color: #000;
    font-weight: 700;
    font-size: 1.2rem;
  }

  .total-valor {
    color: #1a4d7a;
    font-weight: 700;
    font-size: 1.5rem;
  }

  /* Firma */
  .seccion-firma {
    margin: 3rem 0 0.25rem 0;
    display: flex;
    justify-content: space-around;
    gap: 4rem;
  }

  .firma-box {
    width: 300px;
    text-align: center;
    position: relative;
    min-height: 120px;
  }

  .firma-imagen {
    position: absolute;
    bottom: 75px;
    left: 50%;
    transform: translateX(-50%);
    max-width: 200px;
    height: auto;
  }

  .firma-label,
  .firma-nombre {
    position: relative;
    z-index: 0;
  }

  .firma-linea {
    border-top: 2px solid #333;
    margin: 60px 0 0.5rem 0;
    width: 100%;
  }

  .firma-label {
    color: #000;
    font-weight: 600;
    font-size: 0.9rem;
    margin: 0.25rem 0;
  }

  .firma-nombre {
    color: #000;
    font-size: 0.85rem;
    margin: 0.25rem 0;
  }

  /* Pie de página */
  .pie-pagina {
    text-align: center;
    padding-top: 0.5rem;
    border-top: 1px solid #ccc;
    color: #666;
    font-size: 0.9rem;
  }

  .pie-pagina p {
    margin: 0;
  }

  /* ===========================
     ESTILOS PARA IMPRESIÓN
     Optimizado para Oficio Chile Horizontal
     33.02cm x 21.59cm (13in x 8.5in)
  =========================== */
  
  @page {
    size: legal landscape;
    margin: 0.5cm 0.8cm;
  }

  @media print {
    * {
      -webkit-print-color-adjust: exact !important;
      print-color-adjust: exact !important;
    }

    body * {
      visibility: hidden !important;
    }

    .overlay,
    .overlay *,
    .modal-impresion,
    .modal-impresion *,
    .contenido-imprimible,
    .contenido-imprimible * {
      visibility: visible !important;
    }

    .overlay { 
      position: absolute !important;
      left: 0 !important;
      top: 0 !important;
      background: white !important; 
      padding: 0 !important;
      margin: 0 !important;
      display: block !important;
      width: 33.02cm !important;
      height: 21.59cm !important;
    }
    
    .modal-impresion {
      position: static !important;
      box-shadow: none !important;
      background: white !important;
      max-width: none !important;
      width: 33.02cm !important;
      max-height: none !important;
      height: 21.59cm !important;
      overflow: visible !important;
      display: block !important;
      margin: 0 !important;
      padding: 0 !important;
    }

    .no-print { display: none !important; visibility: hidden !important; }
    
    .contenido-imprimible {
      padding: 0 !important;
      color: #000 !important;
      margin: 0 !important;
      width: 100% !important;
      max-width: 100% !important;
    }

    .encabezado {
      border-bottom: 1.5px solid #333;
      margin-bottom: 0.03cm;
      padding: 0.03cm 0.08cm;
      page-break-after: avoid;
      background: #f8f8f8;
    }

    .encabezado h1 {
      font-size: 9pt;
      margin: 0;
      font-weight: 700;
      color: #000 !important;
      line-height: 1;
    }

    .codigo-ot {
      font-size: 8pt;
      font-weight: 600;
      color: #000 !important;
      margin-top: 0.03cm;
      line-height: 1;
    }

    .seccion-info {
      background: #f5f5f5;
      border: none;
      padding: 0.03cm 0.08cm;
      margin-bottom: 0.03cm;
      page-break-inside: avoid;
      font-size: 6.5pt;
    }

    .info-row {
      display: flex;
      justify-content: space-between;
      padding: 0.02cm 0;
      border-bottom: 1px solid #999;
      font-size: 6.5pt;
      line-height: 1;
    }
    
    .info-row:last-child {
      border-bottom: none;
    }

    .label { color: #000 !important; font-weight: 600; }
    .valor { color: #000 !important; }

    .seccion-requerimientos { 
      margin-bottom: 0.02cm;
      page-break-inside: avoid;
      page-break-before: avoid !important;
    }

    .seccion-requerimientos h2 {
      font-size: 8pt;
      margin: 0 0 0.02cm 0;
      padding: 0.02cm 0.06cm;
      background: #e0e0e0;
      border-bottom: 2px solid #333;
      page-break-after: avoid !important;
      color: #000 !important;
      font-weight: 700;
      line-height: 1;
    }

    .tabla-requerimientos-wrapper {
      page-break-before: avoid !important;
      page-break-inside: auto;
      width: 100%;
      margin-bottom: 0.1cm;
      margin-top: 0;
    }

    .tabla-requerimientos {
      width: 100%;
      border-collapse: collapse;
      font-size: 7pt;
      page-break-inside: auto;
      table-layout: fixed;
    }

    .tabla-requerimientos thead th:nth-child(1) { width: 3%; }
    .tabla-requerimientos thead th:nth-child(2) { width: 4%; }
    .tabla-requerimientos thead th:nth-child(3) { width: 16%; }
    .tabla-requerimientos thead th:nth-child(4) { width: 7%; }
    .tabla-requerimientos thead th:nth-child(5) { width: 7%; }
    .tabla-requerimientos thead th:nth-child(6) { width: 5%; }
    .tabla-requerimientos thead th:nth-child(7) { width: 5%; }
    .tabla-requerimientos thead th:nth-child(8) { width: 7%; }
    .tabla-requerimientos thead th:nth-child(9) { width: 23%; }
    .tabla-requerimientos thead th:nth-child(10) { width: 23%; }

    .tabla-requerimientos thead {
      display: table-header-group !important;
      page-break-inside: avoid !important;
      break-inside: avoid !important;
      page-break-after: avoid !important;
      break-after: avoid-page !important;
    }

    .tabla-requerimientos thead th {
      background: #2c3e50 !important;
      color: white !important;
      border: none;
      padding: 0.04cm 0.04cm !important;
      font-size: 7pt;
      font-weight: 700;
      text-align: left;
      vertical-align: middle;
      line-height: 1;
      page-break-inside: avoid !important;
      break-inside: avoid !important;
      -webkit-column-break-inside: avoid !important;
    }

    .tabla-requerimientos thead th:nth-child(4),
    .tabla-requerimientos thead th:nth-child(8) {
      border-right: 1px solid #333 !important;
    }

    .tabla-requerimientos tbody {
      display: table-row-group !important;
    }

    .tabla-requerimientos tbody td {
      color: #000 !important;
      border: none;
      border-bottom: 1px solid #ccc;
      padding: 0.03cm 0.03cm !important;
      font-size: 7pt;
      vertical-align: middle;
      line-height: 1;
      word-wrap: break-word;
      overflow-wrap: break-word;
      page-break-inside: avoid !important;
      break-inside: avoid !important;
      -webkit-column-break-inside: avoid !important;
    }

    .tabla-requerimientos tbody td:nth-child(1),
    .tabla-requerimientos tbody td:nth-child(2),
    .tabla-requerimientos tbody td:nth-child(4),
    .tabla-requerimientos tbody td:nth-child(5),
    .tabla-requerimientos tbody td:nth-child(6),
    .tabla-requerimientos tbody td:nth-child(7),
    .tabla-requerimientos tbody td:nth-child(8) {
      white-space: nowrap;
    }

    .tabla-requerimientos tbody td:nth-child(3),
    .tabla-requerimientos tbody td:nth-child(9),
    .tabla-requerimientos tbody td:nth-child(10) {
      white-space: normal;
      word-wrap: break-word;
      overflow-wrap: break-word;
    }

    .tabla-requerimientos tbody td:nth-child(4),
    .tabla-requerimientos tbody td:nth-child(8) {
      border-right: 1px solid #333 !important;
    }

    .tabla-requerimientos tbody tr:nth-child(even) { background: #fafafa; }
    .tabla-requerimientos tbody tr:nth-child(odd) { background: white; }
    .tabla-requerimientos tbody tr { 
      display: table-row;
      page-break-inside: avoid !important; 
      break-inside: avoid-page !important;
      -webkit-column-break-inside: avoid !important;
      page-break-after: auto !important; 
      page-break-before: auto !important;
      height: auto;
      vertical-align: middle;
    }
    
    .tabla-requerimientos tbody td {
      page-break-inside: avoid !important;
      break-inside: avoid !important;
      -webkit-column-break-inside: avoid !important;
    }

    .centrado { text-align: center; }
    .derecha { text-align: right; }
    .destacado { color: #1a4d7a !important; font-weight: 600; }

    .descripcion-cell {
      font-size: 7.5pt;
      line-height: 1.3;
      word-wrap: break-word;
      overflow-wrap: break-word;
      hyphens: auto;
    }

    .seccion-total {
      page-break-inside: avoid;
      margin-top: 0.1cm;
      background: #e8f4f8;
      border: 2px solid #333;
      padding: 0.06cm 0.15cm;
    }

    .total-row {
      font-size: 7.5pt;
      display: flex;
      justify-content: space-between;
      padding: 0.03cm 0;
    }

    .total-label {
      color: #000 !important;
      font-weight: 700;
    }

    .total-valor {
      color: #1a4d7a !important;
      font-weight: 700;
      font-size: 8.5pt;
    }

    .total-final {
      border-top: 2px solid #333;
      padding-top: 0.3rem !important;
      margin-top: 0.2rem;
    }

    .seccion-firma {
      page-break-inside: avoid;
      margin-top: 0.2cm;
    }

    .seccion-firma {
      page-break-inside: avoid;
      margin-top: 0.2cm;
    }

    .firma-box {
      position: relative;
    }

    .firma-imagen {
      position: absolute !important;
      bottom: 55px !important;
      left: 50% !important;
      transform: translateX(-50%) !important;
      max-width: 3.5cm !important;
      height: auto;
      display: block !important;
    }

    .firma-label,
    .firma-nombre {
      position: relative;
      z-index: 0;
    }

    .firma-label,
    .firma-nombre {
      font-size: 6.5pt;
      color: #000 !important;
    }

    .pie-pagina {
      page-break-inside: avoid;
      font-size: 6pt;
      padding-top: 0.08cm;
      border-top: 1px solid #999;
      color: #000 !important;
      margin-top: 0.1cm;
    }

    .pie-pagina p {
      margin: 0;
    }
  }
</style>
