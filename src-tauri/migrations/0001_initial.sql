PRAGMA foreign_keys = ON;

-- Configuración de empresa (fila única, un solo usuario/equipo)
CREATE TABLE empresa_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    nombre TEXT NOT NULL DEFAULT '',
    nit TEXT NOT NULL DEFAULT '',
    direccion TEXT NOT NULL DEFAULT '',
    telefono TEXT NOT NULL DEFAULT '',
    logo_path TEXT,
    moneda TEXT NOT NULL DEFAULT 'COP',
    formato_fecha TEXT NOT NULL DEFAULT 'DD/MM/YYYY',
    prefijo_comprobante TEXT NOT NULL DEFAULT 'CF',
    siguiente_numero INTEGER NOT NULL DEFAULT 1,
    leyenda_pie TEXT NOT NULL DEFAULT 'Este documento no es una factura electrónica',
    tema TEXT NOT NULL DEFAULT 'claro',
    requiere_pin INTEGER NOT NULL DEFAULT 0,
    pin_hash TEXT,
    permite_stock_negativo INTEGER NOT NULL DEFAULT 0,
    arqueo_activo INTEGER NOT NULL DEFAULT 0,
    creado_en TEXT NOT NULL DEFAULT (datetime('now')),
    actualizado_en TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO empresa_config (id) VALUES (1);

CREATE TABLE categorias (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    categoria_padre_id INTEGER REFERENCES categorias(id) ON DELETE SET NULL,
    creado_en TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE proveedores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nit TEXT,
    razon_social TEXT NOT NULL,
    contacto TEXT,
    telefono TEXT,
    correo TEXT,
    direccion TEXT,
    condiciones_pago TEXT,
    activo INTEGER NOT NULL DEFAULT 1,
    creado_en TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Dinero almacenado en enteros (centavos) para evitar errores de punto flotante
CREATE TABLE productos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sku TEXT NOT NULL UNIQUE,
    codigo_barras TEXT UNIQUE,
    nombre TEXT NOT NULL,
    descripcion TEXT,
    categoria_id INTEGER REFERENCES categorias(id) ON DELETE SET NULL,
    unidad_medida TEXT NOT NULL DEFAULT 'unidad',
    precio_costo INTEGER NOT NULL DEFAULT 0,
    precio_venta INTEGER NOT NULL DEFAULT 0,
    impuesto_pct REAL NOT NULL DEFAULT 0,
    stock_actual INTEGER NOT NULL DEFAULT 0,
    stock_minimo INTEGER NOT NULL DEFAULT 0,
    stock_maximo INTEGER,
    ubicacion TEXT,
    proveedor_principal_id INTEGER REFERENCES proveedores(id) ON DELETE SET NULL,
    estado TEXT NOT NULL DEFAULT 'activo',
    imagen_path TEXT,
    creado_en TEXT NOT NULL DEFAULT (datetime('now')),
    actualizado_en TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_productos_sku ON productos(sku);
CREATE INDEX idx_productos_codigo_barras ON productos(codigo_barras);

CREATE TABLE proveedor_productos (
    proveedor_id INTEGER NOT NULL REFERENCES proveedores(id) ON DELETE CASCADE,
    producto_id INTEGER NOT NULL REFERENCES productos(id) ON DELETE CASCADE,
    PRIMARY KEY (proveedor_id, producto_id)
);

CREATE TABLE compras (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proveedor_id INTEGER REFERENCES proveedores(id) ON DELETE SET NULL,
    numero TEXT NOT NULL,
    fecha TEXT NOT NULL DEFAULT (datetime('now')),
    subtotal INTEGER NOT NULL DEFAULT 0,
    impuestos INTEGER NOT NULL DEFAULT 0,
    total INTEGER NOT NULL DEFAULT 0,
    estado TEXT NOT NULL DEFAULT 'registrada',
    observaciones TEXT
);

CREATE TABLE compra_detalle (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    compra_id INTEGER NOT NULL REFERENCES compras(id) ON DELETE CASCADE,
    producto_id INTEGER NOT NULL REFERENCES productos(id),
    cantidad INTEGER NOT NULL,
    costo_unitario INTEGER NOT NULL,
    subtotal INTEGER NOT NULL
);

-- Kardex: historial de movimientos con saldo acumulado
CREATE TABLE movimientos_stock (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    producto_id INTEGER NOT NULL REFERENCES productos(id),
    tipo TEXT NOT NULL CHECK (tipo IN ('entrada','salida','ajuste','devolucion')),
    cantidad INTEGER NOT NULL,
    costo_unitario INTEGER,
    motivo TEXT,
    referencia_tipo TEXT,
    referencia_id INTEGER,
    fecha TEXT NOT NULL DEFAULT (datetime('now')),
    saldo_resultante INTEGER NOT NULL
);
CREATE INDEX idx_movimientos_producto_fecha ON movimientos_stock(producto_id, fecha);

CREATE TABLE inventario_fisico (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha TEXT NOT NULL DEFAULT (datetime('now')),
    estado TEXT NOT NULL DEFAULT 'abierto',
    observaciones TEXT
);

CREATE TABLE inventario_fisico_detalle (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inventario_fisico_id INTEGER NOT NULL REFERENCES inventario_fisico(id) ON DELETE CASCADE,
    producto_id INTEGER NOT NULL REFERENCES productos(id),
    stock_sistema INTEGER NOT NULL,
    stock_contado INTEGER NOT NULL,
    diferencia INTEGER NOT NULL
);

CREATE TABLE clientes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    documento TEXT,
    nombre TEXT NOT NULL,
    telefono TEXT,
    correo TEXT,
    direccion TEXT,
    creado_en TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE caja_sesiones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha_apertura TEXT NOT NULL DEFAULT (datetime('now')),
    fecha_cierre TEXT,
    monto_apertura INTEGER NOT NULL DEFAULT 0,
    monto_cierre_sistema INTEGER,
    monto_cierre_contado INTEGER,
    diferencia INTEGER,
    observaciones TEXT
);

CREATE TABLE ventas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    numero_comprobante TEXT NOT NULL UNIQUE,
    fecha TEXT NOT NULL DEFAULT (datetime('now')),
    cliente_id INTEGER REFERENCES clientes(id) ON DELETE SET NULL,
    subtotal INTEGER NOT NULL DEFAULT 0,
    descuento_total INTEGER NOT NULL DEFAULT 0,
    impuestos INTEGER NOT NULL DEFAULT 0,
    total INTEGER NOT NULL DEFAULT 0,
    metodo_pago TEXT NOT NULL,
    valor_recibido INTEGER,
    cambio INTEGER,
    estado TEXT NOT NULL DEFAULT 'completada' CHECK (estado IN ('completada','anulada')),
    motivo_anulacion TEXT,
    caja_sesion_id INTEGER REFERENCES caja_sesiones(id) ON DELETE SET NULL
);
CREATE INDEX idx_ventas_fecha ON ventas(fecha);
CREATE INDEX idx_ventas_cliente ON ventas(cliente_id);

CREATE TABLE venta_detalle (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id INTEGER NOT NULL REFERENCES ventas(id) ON DELETE CASCADE,
    producto_id INTEGER NOT NULL REFERENCES productos(id),
    cantidad INTEGER NOT NULL,
    precio_unitario INTEGER NOT NULL,
    descuento_pct REAL NOT NULL DEFAULT 0,
    descuento_valor INTEGER NOT NULL DEFAULT 0,
    impuesto_pct REAL NOT NULL DEFAULT 0,
    subtotal INTEGER NOT NULL
);

CREATE TABLE devoluciones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id INTEGER NOT NULL REFERENCES ventas(id),
    fecha TEXT NOT NULL DEFAULT (datetime('now')),
    motivo TEXT NOT NULL,
    total_devuelto INTEGER NOT NULL
);

CREATE TABLE devolucion_detalle (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    devolucion_id INTEGER NOT NULL REFERENCES devoluciones(id) ON DELETE CASCADE,
    venta_detalle_id INTEGER NOT NULL REFERENCES venta_detalle(id),
    cantidad INTEGER NOT NULL,
    valor INTEGER NOT NULL
);

CREATE TABLE comprobantes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    venta_id INTEGER NOT NULL REFERENCES ventas(id) ON DELETE CASCADE,
    tipo TEXT NOT NULL CHECK (tipo IN ('carta','termico80','imagen')),
    ruta_archivo TEXT NOT NULL,
    generado_en TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE importaciones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entidad TEXT NOT NULL,
    archivo_nombre TEXT NOT NULL,
    fecha TEXT NOT NULL DEFAULT (datetime('now')),
    modo TEXT NOT NULL,
    total_filas INTEGER NOT NULL DEFAULT 0,
    creados INTEGER NOT NULL DEFAULT 0,
    actualizados INTEGER NOT NULL DEFAULT 0,
    omitidos INTEGER NOT NULL DEFAULT 0,
    con_error INTEGER NOT NULL DEFAULT 0,
    archivo_rechazados_path TEXT
);

CREATE TABLE auditoria (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha TEXT NOT NULL DEFAULT (datetime('now')),
    entidad TEXT NOT NULL,
    entidad_id INTEGER,
    accion TEXT NOT NULL CHECK (accion IN ('crear','modificar','anular','eliminar')),
    detalle_json TEXT
);
CREATE INDEX idx_auditoria_entidad ON auditoria(entidad, entidad_id);

CREATE TABLE respaldos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fecha TEXT NOT NULL DEFAULT (datetime('now')),
    ruta_archivo TEXT NOT NULL,
    tipo TEXT NOT NULL CHECK (tipo IN ('manual','automatico')),
    tamano_bytes INTEGER NOT NULL DEFAULT 0
);
