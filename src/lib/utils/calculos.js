/**
 * ============================================================================
 * MÓDULO DE CÁLCULOS MATEMÁTICOS
 * ============================================================================
 * Centraliza TODAS las operaciones matemáticas del sistema de gestión de
 * requerimientos, órdenes de trabajo e informes de pago.
 * 
 * FLUJO DE CÁLCULOS:
 * 1. Crear Requerimiento → precio_total, plazo_total, fecha_limite
 * 2. Recepción → dias_atraso, multa, a_pago
 * 3. Informe de Pago → neto, utilidades, iva, total_pagar (agregación)
 */

// ============================================================================
// CONSTANTES
// ============================================================================

/** Multa fija por día de atraso */
export const MULTA_FIJA_POR_DIA = 7500;

/** Porcentaje de IVA sobre base imponible (19%) */
export const PORCENTAJE_IVA = 0.19;

/** Máximo porcentaje de plazo adicional permitido (50% del plazo base) */
export const PORCENTAJE_PLAZO_ADICIONAL_MAXIMO = 0.5;

/** Decimales para redondeo de montos */
export const DECIMALES_MONTOS = 2;

// ============================================================================
// FUNCIONES AUXILIARES DE REDONDEO
// ============================================================================

/**
 * Redondea un número a decimales específicos
 * @param {number} valor - Valor a redondear
 * @param {number} decimales - Cantidad de decimales (default: 2)
 * @returns {number} Valor redondeado
 */
export function redondear(valor, decimales = DECIMALES_MONTOS) {
  const multiplicador = Math.pow(10, decimales);
  return Math.round(valor * multiplicador) / multiplicador;
}

/**
 * Redondea a entero (sin decimales)
 * @param {number} valor - Valor a redondear
 * @returns {number} Valor redondeado a entero
 */
export function redondearEntero(valor) {
  return Math.round(valor);
}

// ============================================================================
// VALIDACIÓN DE DATOS
// ============================================================================

/**
 * Valida que un valor sea numérico y >= 0
 * @param {*} valor - Valor a validar
 * @param {string} campo - Nombre del campo (para mensaje de error)
 * @returns {number} Valor validado
 * @throws {Error} Si el valor no es válido
 */
export function validarNumeroPositivo(valor, campo = 'valor') {
  const num = Number(valor);
  if (isNaN(num) || num < 0) {
    throw new Error(`${campo} debe ser un número >= 0. Recibido: ${valor}`);
  }
  return num;
}

/**
 * Valida formato de fecha YYYY-MM-DD
 * @param {string} fecha - Fecha a validar
 * @param {string} campo - Nombre del campo (para mensaje de error)
 * @returns {string} Fecha validada
 * @throws {Error} Si la fecha no es válida
 */
export function validarFecha(fecha, campo = 'fecha') {
  if (!fecha) throw new Error(`${campo} es requerida`);
  const regex = /^\d{4}-\d{2}-\d{2}$/;
  if (!regex.test(fecha)) {
    throw new Error(`${campo} debe estar en formato YYYY-MM-DD. Recibido: ${fecha}`);
  }
  return fecha;
}

// ============================================================================
// CÁLCULOS DE PRECIOS
// ============================================================================

/**
 * Calcula precio total de un requerimiento
 * @param {number} cantidad - Cantidad solicitada
 * @param {number} precioUnitario - Precio por unidad
 * @returns {number} Precio total redondeado a 2 decimales
 */
export function calcularPrecioTotal(cantidad, precioUnitario) {
  const cant = validarNumeroPositivo(cantidad, 'cantidad');
  const precio = validarNumeroPositivo(precioUnitario, 'precioUnitario');
  return redondear(cant * precio);
}

/**
 * Recalcula precio_total cuando cambian cantidad o precio_unitario
 * @param {Object} requerimiento - Objeto con campos cantidad y precio_unitario
 * @returns {number} Precio total calculado
 */
export function recalcularPrecioTotal(requerimiento) {
  return calcularPrecioTotal(
    requerimiento.cantidad || 0,
    requerimiento.precioUnitario || 0
  );
}

// ============================================================================
// CÁLCULOS DE FECHAS Y PLAZOS
// ============================================================================

/**
 * Calcula plazo total sumando plazo base + plazo adicional
 * @param {number} plazo - Plazo base en días
 * @param {number} plazoAdicional - Plazo adicional en días (default: 0)
 * @returns {number} Plazo total en días
 */
export function calcularPlazoTotal(plazo, plazoAdicional = 0) {
  const p = validarNumeroPositivo(plazo || 0, 'plazo');
  const pa = validarNumeroPositivo(plazoAdicional || 0, 'plazoAdicional');
  return p + pa;
}

/**
 * Calcula la fecha límite sumando días al fecha_inicio
 * @param {string} fechaInicio - Fecha en formato YYYY-MM-DD
 * @param {number} plazoTotal - Días de plazo
 * @returns {string|null} Fecha límite en formato YYYY-MM-DD o null
 */
export function calcularFechaLimite(fechaInicio, plazoTotal) {
  if (!fechaInicio || !plazoTotal) return null;
  
  validarFecha(fechaInicio, 'fechaInicio');
  const dias = validarNumeroPositivo(plazoTotal, 'plazoTotal');
  
  const [año, mes, dia] = fechaInicio.split('-').map(Number);
  const fecha = new Date(año, mes - 1, dia);
  fecha.setDate(fecha.getDate() + dias);
  
  const d = String(fecha.getDate()).padStart(2, '0');
  const m = String(fecha.getMonth() + 1).padStart(2, '0');
  const a = fecha.getFullYear();
  
  return `${a}-${m}-${d}`;
}

/**
 * Calcula días de atraso entre fecha de recepción y fecha límite
 * @param {string} fechaRecepcion - Fecha de recepción YYYY-MM-DD
 * @param {string} fechaLimite - Fecha límite YYYY-MM-DD
 * @returns {number} Días de atraso (positivo = atraso, negativo = adelanto, 0 = a tiempo)
 */
export function calcularDiasAtraso(fechaRecepcion, fechaLimite) {
  if (!fechaRecepcion || !fechaLimite) return 0;
  
  validarFecha(fechaRecepcion, 'fechaRecepcion');
  validarFecha(fechaLimite, 'fechaLimite');
  
  const [añoLim, mesLim, diaLim] = fechaLimite.split('-').map(Number);
  const [añoRec, mesRec, diaRec] = fechaRecepcion.split('-').map(Number);
  
  const fechaLim = new Date(añoLim, mesLim - 1, diaLim);
  const fechaRec = new Date(añoRec, mesRec - 1, diaRec);
  
  return Math.floor((fechaRec - fechaLim) / (1000 * 60 * 60 * 24));
}

/**
 * Calcula máximo de días adicionales permitidos (50% del plazo base)
 * @param {number} plazo - Plazo base en días
 * @returns {number} Máximo días adicionales permitidos
 */
export function calcularDiasMaximoPlazoAdicional(plazo) {
  const p = validarNumeroPositivo(plazo || 0, 'plazo');
  return Math.floor(p * PORCENTAJE_PLAZO_ADICIONAL_MAXIMO);
}

// ============================================================================
// CÁLCULOS DE MULTAS Y PAGOS
// ============================================================================

/**
 * Calcula multa por atraso usando la fórmula:
 * max(dias_atraso × 7500, dias_atraso × (precio_total / plazo_total))
 * @param {number} precioTotal - Precio total del requerimiento
 * @param {number} diasAtraso - Días de atraso (puede ser negativo)
 * @param {number} plazoTotal - Plazo total en días
 * @returns {number} Multa calculada (0 si no hay atraso)
 */
export function calcularMulta(precioTotal, diasAtraso, plazoTotal = 0) {
  // Si no hay atraso o es adelanto, no hay multa
  if (diasAtraso <= 0) return 0;
  
  const precio = validarNumeroPositivo(precioTotal || 0, 'precioTotal');
  const dias = validarNumeroPositivo(diasAtraso, 'diasAtraso');
  const plazo = validarNumeroPositivo(plazoTotal || 0, 'plazoTotal');
  
  // Opción 1: Multa fija por día
  const opcion1 = dias * MULTA_FIJA_POR_DIA;
  
  // Opción 2: Multa proporcional al precio y plazo
  const opcion2 = plazo > 0 ? dias * (precio / plazo) : 0;
  
  return redondearEntero(Math.max(opcion1, opcion2));
}

/**
 * Calcula monto a pagar restando multa del precio total
 * @param {number} precioTotal - Precio total
 * @param {number} multa - Multa aplicada
 * @returns {number} Monto a pagar (puede ser negativo si multa > precio)
 */
export function calcularAPago(precioTotal, multa) {
  const precio = validarNumeroPositivo(precioTotal || 0, 'precioTotal');
  const m = validarNumeroPositivo(multa || 0, 'multa');
  return precio - m;
}

/**
 * Calcula campos financieros de una línea de requerimiento
 * @param {number} aPago - Monto a pago (después de multas)
 * @param {number} porcentajeUtilidades - % de utilidades (ej: 0.25 para 25%)
 * @returns {Object} { utilidades, iva, totalLinea }
 */
export function calcularLineaRequerimiento(aPago, porcentajeUtilidades) {
  const utilidades = redondearEntero(aPago * porcentajeUtilidades);
  const baseIva = aPago + utilidades;
  const iva = redondearEntero(baseIva * PORCENTAJE_IVA);
  const totalLinea = aPago + utilidades + iva;
  
  return { utilidades, iva, totalLinea };
}

// ============================================================================
// CÁLCULOS DE INFORMES DE PAGO
// ============================================================================

/**
 * Calcula campos financieros de un informe de pago (AGREGACIÓN)
 * @param {Array} requerimientos - Array con campos: aPago, utilidades, iva, totalLinea
 * @returns {Object} { neto, utilidades, iva, totalPagar }
 */
export function calcularCamposInforme(requerimientos) {
  const neto = requerimientos.reduce((sum, req) => sum + (Number(req.aPago) || 0), 0);
  const utilidades = requerimientos.reduce((sum, req) => sum + (Number(req.utilidades) || 0), 0);
  const iva = requerimientos.reduce((sum, req) => sum + (Number(req.iva) || 0), 0);
  const totalPagar = requerimientos.reduce((sum, req) => sum + (Number(req.totalLinea) || 0), 0);
  
  return { neto, utilidades, iva, totalPagar };
}

/**
 * Recalcula campos financieros completos de un requerimiento
 * Útil cuando se modifica cantidad, precio, fecha_recepcion, etc.
 * @param {Object} requerimiento - Requerimiento con todos los campos necesarios
 * @returns {Object} Campos actualizados { precio_total, dias_atraso, multa, a_pago }
 */
export function recalcularCamposFinancierosRequerimiento(requerimiento) {
  const precio_total = calcularPrecioTotal(
    requerimiento.cantidad || 0,
    requerimiento.precioUnitario || 0
  );
  
  const dias_atraso = calcularDiasAtraso(
    requerimiento.fechaRecepcion,
    requerimiento.fechaLimite
  );
  
  const multa = calcularMulta(
    precio_total,
    dias_atraso,
    requerimiento.plazoTotal || 0
  );
  
  const a_pago = calcularAPago(precio_total, multa);
  
  return { precio_total, dias_atraso, multa, a_pago };
}

// ============================================================================
// FUNCIONES DE FORMATEO
// ============================================================================

/**
 * Formatea número con separadores de miles (formato chileno), sin decimales
 * @param {number} valor - Número a formatear
 * @returns {string} Número formateado (ej: "1.234.567")
 */
export function formatearNumero(valor) {
  return new Intl.NumberFormat('es-CL').format(Math.round(Number(valor) || 0));
}

/**
 * Formatea días de atraso asegurando valor >= 0
 * @param {number} dias - Días de atraso
 * @returns {number} Días formateados (mínimo 0)
 */
export function formatearDiasAtraso(dias) {
  return Math.max(0, Number(dias) || 0);
}

/**
 * Formatea monto como moneda chilena
 * @param {number} valor - Valor a formatear
 * @returns {string} Valor formateado con símbolo $ (ej: "$1.234.567")
 */
export function formatearMoneda(valor) {
  return `$${formatearNumero(valor)}`;
}

// ============================================================================
// FUNCIONES AUXILIARES
// ============================================================================

/**
 * Calcula el monto total de requerimientos seleccionados
 * @param {Array} requerimientos - Array de requerimientos
 * @param {Set} seleccionados - Set con IDs de requerimientos seleccionados
 * @returns {number} Suma de precio_total de los seleccionados
 */
export function calcularMontoTotalSeleccionados(requerimientos, seleccionados) {
  return requerimientos
    .filter(r => seleccionados.has(r.id))
    .reduce((sum, r) => sum + (Number(r.precioTotal) || 0), 0);
}

/**
 * Calcula suma total de un campo específico en un array
 * @param {Array} items - Array de objetos
 * @param {string} campo - Nombre del campo a sumar
 * @returns {number} Suma total del campo
 */
export function calcularSumaCampo(items, campo) {
  return items.reduce((sum, item) => sum + (Number(item[campo]) || 0), 0);
}

/**
 * Convierte string a número de manera segura
 * @param {*} valor - Valor a convertir
 * @param {number} defaultValue - Valor por defecto si conversión falla (default: 0)
 * @returns {number} Número convertido o valor por defecto
 */
export function aNumeroSeguro(valor, defaultValue = 0) {
  const num = Number(valor);
  return isNaN(num) ? defaultValue : num;
}
