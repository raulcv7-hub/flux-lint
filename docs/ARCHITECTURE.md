# GUÍA SOBRE DECISIÓN ARQUITECTÓNICA

**Propósito:** Este documento actúa como el "Framework de Referencia" para el equipo de ingeniería. Define no solo *qué* tecnologías usamos, sino *por qué* y *cómo* se estructuran.

---

## 0. NIVEL PRE-REQUISITOS: Restricciones del Mundo Real

* **Performance:** El usuario espera feedback en < 200ms para proyectos pequeños. -> *Rust + Paralelismo obligatorios.*
* **Entorno:** Ejecución local (CLI). -> *Sin costes de nube, pero dependemos de la CPU del usuario.*
* **Fiabilidad:** Cero falsos positivos tolerables en sintaxis. -> *Dependencia dura de Tree-sitter.*

---

## 1. NIVEL SISTEMA: Topología Física (The Shape)

| Topología | Selección | Justificación para Audit |
| :--- | :--- | :--- |
| **Monolito Modular** | **[X]** | Es un binario único (`.exe`/bin). No tiene sentido distribuir microservicios para una herramienta de terminal. La modularidad será interna (Crates/Modules), no física. |
| **Microservicios** | `[ ]` | Añadiría latencia de red innecesaria. |

---

## 2. NIVEL DATOS: Estrategia de Estado (Data Scope)

### 2.1. Propiedad del Dato

* **In-Memory Read-Only:** Audit no posee los datos (el código fuente), solo los lee prestados del disco.
* **Transient State:** El estado (AST, Métricas) vive solo durante la ejecución del proceso.

### 2.2. Consistencia

* **[X] Atomicidad (Thread-Safety):** Al usar `rayon` para paralelismo, usamos estructuras de datos inmutables o contadores atómicos (`AtomicUsize`) para métricas globales.

### 2.3. Modelo de Persistencia

* **[X] N/A:** No hay base de datos. La "persistencia" es el reporte final (JSON/Markdown).

---

## 3. NIVEL ESTRUCTURAL: Organización Lógica (Logical Scope)

| Patrón | Selección | Justificación |
| :--- | :--- | :--- |
| **Hexagonal (Ports & Adapters)** | **[X]** | Vital para testear. <br> - **Core:** Reglas de análisis (puros). <br> - **Puertos:** `SourceProvider` (Filesystem), `Reporter` (Stdout). <br> Esto nos permite testear reglas inyectando código en memoria sin tocar el disco. |
| **Pipeline** | **[X]** | El flujo es lineal: `Input -> Walk -> Parse -> Analyze -> Report`. |

---

## 4. NIVEL IMPLEMENTACIÓN: Patrones Tácticos (Code Scope)

* **Comunicación entre Módulos:**
  * **[X] Data Parallelism:** Modelo Fork-Join (Map-Reduce) usando `rayon`. Mapeamos archivos a resultados y reducimos a un reporte global.
* **Manejo de Errores:**
  * **[X] Result Pattern:** Uso estricto de `Result<T, E>`.
  * **[X] Error Context:** Uso de `anyhow` para aplicaciones y `thiserror` para librerías. *Regla:* Nunca hacer `panic!` en el hilo principal.
* **Principios Rectores:**
  * **Open/Closed Principle:** Añadir un nuevo lenguaje (ej: Go) no debe requerir modificar el motor core, solo añadir un adaptador.
  * **Functional Core, Imperative Shell:** La lógica de detección de smells debe ser pura (sin efectos secundarios).

---

## 5. NIVEL OPERATIVO: Arquitectura de Despliegue (Ops Scope)

### 5.1. Estrategia de Distribución

* **[X] Binary Release:** Compilación estática (`musl` si es posible en Linux) para evitar "DLL hell".

### 5.2. Observabilidad

* **Logs:** `tracing` crate. Niveles: `INFO` (progreso usuario), `DEBUG` (tiempos de parseo), `TRACE` (tokens AST).
* **Métricas:** Tiempos de ejecución por fase (Parseo vs Análisis) mostrados al final si flag `--debug`.

---

## 6. ATRIBUTOS DE CALIDAD: Los Trade-offs

> **Prioridades Críticas:**

1. **[X] Performance (Throughput):** Procesar miles de líneas por segundo. Sacrificamos simplicidad de código (uso de hilos) por velocidad.
2. **[X] Fiabilidad (Accuracy):** Preferimos no reportar nada a reportar un falso positivo. Sacrificamos "sugerencias inteligentes" por "hechos estructurales".

---

## 7. GOBERNANZA Y EVOLUCIÓN

Para añadir una nueva **Regla/Smell**:

1. Definir caso de prueba (código con el error y código sin él).
2. Escribir la Query Tree-sitter `.scm` o la lógica Rust.
3. Validar contra falsos positivos.

---

### Anexo: Glosario de Tecnologías Elegidas (Stack)

* **Lenguaje:** Rust (Edición 2021).
* **CLI Framework:** `clap` (v4).
* **Parallel Engine:** `rayon`.
* **Parser Core:** `tree-sitter`.
* **Query Language:** S-expressions (Scheme) para Tree-sitter.
