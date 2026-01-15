export function validarRequerimiento(data) {
  const errores = {};
  
  // Obligatorios: contexto, valor y plazo
  if (!data.jardinCodigo) errores.jardin = 'Seleccione un jardín';
  if (!data.recinto || data.recinto.trim() === '') errores.recinto = 'Seleccione una zona/recinto';
  if (!data.item) errores.partida = 'Seleccione una partida';
  if (!data.cantidad || data.cantidad <= 0) errores.cantidad = 'Ingrese cantidad válida';
  if (!data.fechaInicio) errores.fechaInicio = 'Seleccione fecha de inicio';
  if (!data.plazoDias) errores.plazo = 'Seleccione plazo en días';
  
  return Object.keys(errores).length === 0 ? null : errores;
}
